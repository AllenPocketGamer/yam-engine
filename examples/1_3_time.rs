use yam::legion::*;
use yam::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_system_process(println_time_system())
        // .add_system_process(diagnostic_time_cost_system(DiagnosticTimer::new()))
        // //                                              ^^^^^^^^^^^^^^^^^^^^^^+-------------+
        // //                                                                                  |
        // // Construct the system-local variable and move it into the system. <---------------+
        .into_app_builder()
        .build()
        .run();

    Ok(())
}

// Get `Time` from `Resources`.
//
// `Time` is used to record the time information of `AppStage`, different `AppStage`
// maintain their own `Time`.
#[system]
fn println_time(#[resource] time: &Time) {
    // Format and print the `Time` information is too time consuming, so spawn a new thread
    // and move the code into it to prevent the time consuming.
    let cpy = *time;
    std::thread::spawn(move || {
        println!("{}", cpy);
    });
}

// You CANNOT create or modify `Time` by yourself, but you can access the useful information
// from `Time`.
//
// If you want to diagnose the time cost of a piece of code, `DiagnosticTimer` may be able to
// be helpful.
#[system]
fn diagnostic_time_cost(#[state] d_timer: &mut DiagnosticTimer)
//                      ^^^^^^^^+-----------------------------------------------------------+
//                                                                                          |
// a macro-attribute from `legion` which to store system-local variable. <------------------+
{
    d_timer.start_record();

    // Sleep 10ms.
    std::thread::sleep(std::time::Duration::from_millis(10));

    d_timer.stop_record();

    println!("{}", d_timer);
}
