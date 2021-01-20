use app::{App, AppBuildError, AppBuilder, AppRunError, AppStage, AppStageBuilder};
use yamengine::*;

fn main() -> Result<(), AppBuildError> {
    let stage_builder = AppStage::builder();
    
    App::builder()
        .add_stage_builder(stage_builder)?
        .create_stage_builder(String::from("first"), 60)?
        .into_app_builder()
        .build()
        .run();
    
    Ok(())
}