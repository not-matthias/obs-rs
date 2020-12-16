use graphic_offsets::GraphicOffsetsError;
use inject_helper::InjectHelperError;

#[derive(Debug)]
pub enum ObsError {
    ProcessNotFound,
    Inject(InjectHelperError),
    LoadGraphicOffsets(GraphicOffsetsError),
    CreatePipe,
    CreateMutex,
    CreateFileMapping,
    CreateDevice,
    OpenSharedResource,
    CreateTexture,
    MapSurface,
}
