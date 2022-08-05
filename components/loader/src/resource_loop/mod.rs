/// A loop running on a separate thread that handle all resource fetching requests
/// from both the main process & the render process. Currently, it handles requests
/// for main process & render process separately but in the end, it should sit only
/// in the main process & the render process should not have a resource loop but go
/// through the main process to request for resource.
pub mod error;
pub mod request;
pub mod resource_loop;

pub use resource_loop::*;