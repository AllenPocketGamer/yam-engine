# yam-engine的重构设计

## yam-engine当前的大模块

---

* AppBuilder: 负责创建App
* App: 负责执行游戏循环, 管理用户提供的资源, 调度用户提供的行为, 向用户提供内置的资源
* Window: 负责维护窗口和其事件循环, 在另一个线程中执行
* Input: App向外提供的资源, 描述了使用者对输入设备的操作
* Settings: App向外提供的资源, 用于对App进行配置, 如window的属性, 设置循环定时器

## app重构, stage

---

App提供新的功能: Stage; Stage有自己的名字和更新频率, 在Build的时候可以创建Stage

```rust
fn main() {
  App::build()
    .add_stage(unique_name, frequency)
    .add_startup_system_to_stage(system, stage_name)
    .add_update_system_to_stage(system, stage_name)
    .add_startup_system(system) // add to default stage
    .add_update_system(system)  // add to default stage
    .finish()
    .run();
}
```

在运行时增删改查Stage

```rust
#[system]
fn add_stage(#[resource] settings: &mut Settings) {
    let stage = AppStage::new(unique_name, frequency);
    stage.add_startup_system(...);
    stage.add_update_system(...);
    settings.app.add_stage(stage);
}

#[system]
fn remove_stage(#[resource] settings: &mut Settings) {
    settings.app.remove_stage(stage_name);
}

#[system]
fn change_stage(#[resource] settings: &mut Settings) {
    let stage = settings.app.stage_mut(stage_name);
    stage.frequency = 10;
    stage.name = String::from("Nothing");
}

#[system]
fn iter_stage(#[resource] settings: &mut Settings) {
    let stages = settings.app.stages();
    let stages_mut = settings.app.stages_mut();

    // do something query...
}
```

Stage特性描述

1. Stage之间是串行执行
2. Stage有start, update, destroy
    * start只在程序开始或Stage插入时执行一次
    * update被循环调用
    * destroy在程序结束或Stage被移除时执行一次

```rust
fn main() {
    // create stage by stagebuilder
    let stage = AppStageBuilder::new(stage_name, stage_frequency)
        .add_system_startup(...)
        .add_system_update(...)
        .add_system_destroy(...)
        .add_thread_local_fn_startup(...)
        .add_thread_local_fn_update(...)
        .add_thread_local_fn_destroy(...)
        .add_thread_local_system_startup(...)
        .add_thread_local_system_update(...)
        .add_thread_local_system_destroy(...)
        .finish();

    // create empty stage by AppStage
    let stage = AppStage::new(stage_name, stage_frequency);

    // create default stage(name: "default", frequency: 60)
    let stage = AppStage::default();

    // or...
    let stage = AppStageBuilder::default().finish();

    // or by builder
    let stage = AppStage::build(stage_name, stage_frequency).finish();
}
```

通过AppSettings来读写App的状态

```rust
#[system]
fn setttings(#[resource] settings: &mut AppSettings) {
    // Getter
    let busy_stage: &AppStage = settings.busy_stage(stage_name);
    let busy_stages: Iter<AppStage> = settings.busy_stage_iter();

    let spare_stage: &AppStage = settings.spare_stage(stage_name);
    let spare_stages: Iter<AppStage> = settings.spare_stages();

    let spare_stage: AppStage = settings.take_spare_stage(stage_name);
    let spare_stages: Vec<AppStage> = settings.take_spare_stages();

    // Push stage to work
    settings.make_stage_work(stage);
    settings.make_spare_stage_work(spare_stage_name);

    // Push stage to rest
    settings.make_stage_rest(stage);
    settings.make_busy_stage_rest(busy_stage_name);

    // Change stage frequency
    settings.set_stage_frequency(frequency);
}
```
