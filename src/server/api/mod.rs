use crate::server::application::resource::ResourceService;

pub mod resource;

pub struct AppState {
    pub resource_service: ResourceService,
}
