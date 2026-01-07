use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sigv4::http_request::{SignableBody, SignableRequest, SigningSettings, sign};
use aws_sigv4::sign::v4::{SigningParams, calculate_signature, generate_signing_key};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::any;
use axum::{Router, middleware};
use http::StatusCode;
use regex::Regex;
use s3s::S3;
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

#[derive(Default)]
pub struct MockS3;

#[async_trait]
impl S3 for MockS3 {}

#[tokio::main]
async fn main() {
    // 业务路由
    let app = Router::new()
        .route("/{*wildcard}", any(hello))
        // 在这里挂载 SigV4 中间件
        .layer(middleware::from_fn(verify_aws_sigv4));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub async fn hello() -> &'static str {
    "Hello, World!"
}

// 模拟获取 Secret Key
async fn get_secret_by_ak(ak: &str) -> Option<String> {
    if ak == "test" {
        Some("test".to_string())
    } else {
        None
    }
}

pub async fn verify_aws_sigv4(req: Request, next: Next) -> Result<Response, StatusCode> {
    // 2. 提取必要的 Header
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let amz_date = req
        .headers()
        .get("x-amz-date")
        .or_else(|| req.headers().get("date"))
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 3. 解析 Authorization Header
    // 格式: AWS4-HMAC-SHA256 Credential=AKIA/20230101/region/service/aws4_request, SignedHeaders=..., Signature=...
    let re = Regex::new(r"AWS4-HMAC-SHA256 Credential=([^/]+)/([^/]+)/([^/]+)/([^/]+)/aws4_request, SignedHeaders=([^,]+), Signature=([a-f0-9]+)").unwrap();
    println!("Auth:{}", auth_header);
    let caps = re.captures(auth_header).ok_or(StatusCode::UNAUTHORIZED)?;

    let access_key_id = &caps[1];
    let date_stamp = &caps[2]; // YYYYMMDD
    let region = &caps[3];
    let service = &caps[4];
    let signed_headers_str = &caps[5];
    let provided_signature = &caps[6];

    // 7. 构建 Canonical Request
    let canonical_uri = req.uri().path(); // 假设路径已标准化
    let canonical_querystring = normalize_query_string(req.uri().query().unwrap_or(""));

    // 处理 Canonical Headers (需要排序)
    let headers_to_sign: Vec<&str> = signed_headers_str.split(';').collect();
    let mut canonical_headers = String::new();
    for &h_name in &headers_to_sign {
        let h_val = req
            .headers()
            .get(h_name)
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_str()
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        // 简单处理：转小写，去除首尾空格，压缩中间空格
        canonical_headers.push_str(&format!("{}:{}\n", h_name.to_lowercase(), h_val.trim()));
    }

    // 计算 Body Hash
    let payload_hash = "UNSIGNED-PAYLOAD".to_string();

    // 如果客户端发送了 x-amz-content-sha256，也可以在这里校验它是否匹配 payload_hash

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        req.method().as_str(),
        canonical_uri,
        canonical_querystring,
        canonical_headers,
        signed_headers_str,
        payload_hash
    );

    println!("{}", canonical_request);

    let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));

    // 8. 构建 String to Sign
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}/{}/{}/aws4_request\n{}",
        amz_date, date_stamp, region, service, canonical_request_hash
    );

    // let re = Regex::new(
    //     r"Credential=([^/]+)/([^/]+)/([^/]+)/([^/]+)/aws4_request.*Signature=([a-f0-9]+)",
    // )
    // .unwrap();
    // let caps = re.captures(auth_header).ok_or(StatusCode::UNAUTHORIZED)?;

    // let access_key = &caps[1];
    // let _date_stamp = &caps[2];
    // let region = &caps[3];
    // let service = &caps[4];
    // let client_signature = &caps[5];

    // 4. 获取对应的 Secret Key
    let secret_key = get_secret_by_ak(access_key_id)
        .await
        .ok_or(StatusCode::FORBIDDEN)?;

    // 5. 确定签名时间

    // 解析时间格式 ISO8601 (YYYYMMDD'T'HHMMSS'Z')
    let req_time = parse_amz_date(amz_date).ok_or(StatusCode::UNAUTHORIZED)?;

    // 6. 校验时间偏差 (防止重放攻击，例如允许 ±5 分钟)
    if SystemTime::now()
        .duration_since(req_time)
        .unwrap_or(Duration::ZERO)
        > Duration::from_secs(300)
    {
        return Err(StatusCode::FORBIDDEN); // Request too old
    }

    // 7. 使用 aws-sigv4 官方库重新计算签名

    let derived_key = generate_signing_key(&secret_key, req_time, region, service);
    let signature = calculate_signature(derived_key, string_to_sign.as_bytes());

    // 8. 比对签名
    println!("sign:{}", signature);
    if provided_signature != signature {
        return Err(StatusCode::UNAUTHORIZED); // 签名不匹配
    }

    Ok(next.run(req).await)
}

// 简单的 ISO8601 解析器
fn parse_amz_date(date_str: &str) -> Option<SystemTime> {
    // 格式: 20231010T120000Z
    use time::{PrimitiveDateTime, format_description::well_known::Iso8601};
    let format =
        time::format_description::parse("[year][month][day]T[hour][minute][second]Z").ok()?;
    let dt = PrimitiveDateTime::parse(date_str, &format).ok()?;
    Some(dt.assume_utc().into())
}

// AWS 要求 Query 参数必须按 key 字母顺序排序，并且编码
fn normalize_query_string(query: &str) -> String {
    if query.is_empty() {
        return String::new();
    }
    // 解析 -> 排序 -> 编码 -> 重组
    let mut pairs: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    // 按 key 排序
    pairs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in pairs {
        serializer.append_pair(&k, &v);
    }
    serializer.finish()
}
