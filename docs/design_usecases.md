* 运行空项目
   ```Rust
   fn main() {
      App::build().run();
   }
   ```
* 运行60fps的逻辑循环
   ```Rust
   fn main() {
      App::build()
         .add_loop_system(loop_system())
         .run();
   }

   #[system(for_each)]
   fn loop(component01, component02, ...) {
      // do something
   }
   ```

* 生成一个窗口
   ```Rust
   fn main() {

   }
   ```

* 回应输入事件
   ```Rust
   fn main() {

   }
   ```

* 绘制一个精灵
   ```Rust
   fn main() {
       
   }
   ```