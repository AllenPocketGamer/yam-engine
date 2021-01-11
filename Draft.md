# 如何完善YamEngine的基本循环系统

## 实现UpdateLayer

在构建时, 共有最多8层UpdateLayer, 每层Layer有`{name, frequency, systems}`的基本属性;

在运行时, 每层Layer有`{name, frequency, timer, scheduler}`的基本属性; scheduler不该暴露给用户, timer不能让用户写, frequency可由用户读写, 但要通过函数, name可由用户读取

如何设计数据结构来CRUD Update Layer是一项要完成的工作;

在配置完毕运行时如何检索UpdateLayer

```rust
#|[system]
fn iter_update_layers(#[resources] app: &App) {
    for update_layer in app.get_update_layers() {
        let layer_name = update_layer.get_name();
        let frequency = update_layer.get_frequency();
        let timer = update_layer.get_timer();

        println!("update layer [name : {}, frequency: {}, timer: {}]", layer_name, frequency, timer);
    }
}

#[system]
fn change_update_layers(#[resource] app: &App) {
    for update_layer in app.get_update_layers() {
        update_layer.set_frequency("layer_name", 90);
    }
}
```

显然`UpadteLayer`的所有权不应该交给Resources, 所有权应依然保存在`run()`函数中;

下面讨论