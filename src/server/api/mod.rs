use serde::{Deserialize, Serialize};

use crate::server::application::resource::ResourceService;

pub mod resource;

pub struct AppState {
    pub resource_service: ResourceService,
}

#[derive(Serialize, Deserialize)]
pub struct Resp<T> {
    pub code: u32,
    pub data: Option<T>,
    pub message: Option<String>
}

impl <T> Resp<T> {
    
    pub fn ok(data: T) -> Self{
        Resp { code: 200, data: Some(data), message: None }
    }

    pub fn err(message: String) -> Self{
        Resp { code: 500, data: None, message: Some(message) }
    }

}

pub fn err_resp(message: String) -> Resp<()> {
    Resp { code: 500, data: None, message: Some(message) }
}
