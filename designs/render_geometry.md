# 几何绘制设计

## 几何绘制分类

1. 有界几何: 线段/圆/正方形/等边三角

## 几何绘制样式

用来决定几何如何呈现.

1. 边框: 风格: 无/实线/虚线/航线/警示线; 特性: 静态/动态; 外观: 颜色
2. 内里: 风格: 无/实心/波点; 外观: 颜色

## 应用

先来统计`线绘制`在游戏中可能的应用:

* 线段: 全局类, 例如网格; 指示型工作, 例如瞄准线
* 曲线: 路径指示, 道路绘制

再来统计`面绘制`在游戏中可能的应用:

* 所有实心几何: 用于指代不同的运动/非运动单位, 例如人类/子弹/建筑...
* 部分空心几何: 指示型工作, 例如选中框, 单位指示框(用红绿实边空心圆来区分敌我)

## 伪代码

### 网格绘制

```rust
// 从逻辑上, 整个世界会被网格划为横平竖直的一块块正方形;
// 但世界可以近乎认为无限, 所以若要把世界完全划分, 所需
// 的直线是巨量的, 这需要避免!
// 
// 最简单的思路就是在摄像范围内绘制网格, 摄像范围外不管;
// 通过<Transform2D, Camera2D>获得摄像机位置, 范围
// (在世界坐标系), 根据上述两个数据来决定绘制多少直线,
// 绘制在哪.
//
// 显然, 在此处应用中, 命令式绘制要比声明式绘制好的多;
// 但如何合理划分命令式和声明式, 是一个需要智慧的地方.
#[system(for_each)]
fn draw_grid(transform2d: &Transform2D,
             camera2d: &Camera2D,
             #[resource]: render2d: &mut Render2D,
) {
    // 正方形网格的边长为16.0.
    const SIZE: f32 = 16.0;
    
    let cpos = transform2d.position;
    let csize = Vector2::new(transform2d.scale.x * camera2d.width,
                             transform2d.scale.y * camera2d.height
                            );
    
    // 调用Render2D::draw_line(&mut self, st: Vector2, ed: Vector2)
    // 来绘制线段, 形成网格.
}
```

### 结构体设计(请无视内存布局)

```rust
// 用于表示图元的类型
enum Geometry {
    Quad = 0,
    RQuad,
    Circle,
    ETriangle,
    // Line,
}

enum BorderType {
    None = 0,   // 没有边框
    Solid,      // 实边框
    Dash,       // 虚边框
    Navi,       // 导航边框
    Warn,       // 警示边框
}

// 边框样式结构体
struct BorderStyle {
    b_type: BorderType,
    is_dynamic: bool,
    color: Rgba,
}

enum InnerType {
    None = 0,   // 空心
    Solid,      // 实心
    Dither,     // 抖动
}

enum InnerStyle {
    i_type: InnerType,
    is_dynamic: bool,
    color: Rgba,
}
```

### 声明式绘制

```rust
#[system]
fn init(cmd: &mut CommandBuffer) {
    // 绘制一个`图元`, 默认样式为无边框实心pink色
    cmd.push((Transform2D::default(), Geometry::Circle));

    // 绘制一个`图元`, 携带`BorderStyle`
    cmd.push((Transform2D::default(), Geometry::Circle, BorderStyle::default()));

    // 绘制一个`图元`, 携带`InnerStyle`
    cmd.push((Transform2D::default(), Geometry::Circle, InnerStyle::default()));

    // 绘制一个`图元`, 同时携带`BorderStyle`和`InnerStyle`
    cmd.push((Transform2D::default(), Geometry::Circle, BorderStyle::default(),
        InnerStyle::default()
    ));
}
```

### 命令式绘制

```rust
#[system]
fn draw(#[resource] r2d: &mut Render2D) {
    r2d.draw_quad(lb, rt, bstyle, istyle);
    r2d.draw_rquad(lb, rt, bstyle, istyle);
    r2d.draw_circle(centre, radius, bstyle, istyle);
    r2d.draw_etriangle(centre, radius, bstyle, istyle);
}
```

## 实现

着色过程显然是放在GPU端进行的, 但是放在`fragment shader`还是`compute shader`是值得商榷的.

不过最先的, 是要讨论算法的设计.

最简单的算法, 做一张屏幕网格, 让每个片元逐图元计算; 如果图元的个数较少(1_000以下), 那么该方法还可以接受, 但量一旦上来, 例如百万级别, 效率方面就不可接受, 因为粗算需要的算力O = W * H * N, 若W = 1920, H = 1080, N为10^6量级, 那么O便是10^12量级, 这个量级完全不可能做实时渲染的.

好在大多数图元都是有界的, 可以利用传统的v-f着色器, 将图元视作程序纹理!

为了减少冗余信息, 把完整的`图元`切分成`图元类型`, `边框样式`, `内部样式`这三块, 使用时可以用组合的方式按需取用.

有的时候, 希望使用直接的命令在世界坐标系中画`图元`, 例如`draw_circle(cetre, radius, bstyle, istyle)`之类的函数; 这些命令会在后端生成图元数据, 传递给GPU(这步跟声明式一样); 唯一不同的是, 声明式会在每帧重新传递数据到GPU端, 而命令式会有所选择.

命令式绘制一般使用的场景是辅助性工作, 例如绘制网格, 类UI提示性工作(例如导航线, 禁止线), 甚至可能是具体的对象, 例如道路, 障碍物等等; 像这种一旦绘制就几乎不会更新的图元, 可以用静态的方式一次性直接上传, 而不用每帧更新, 从而避免io对性能的损耗; 但这属于高级特性, 实现的时间点值得商榷.

还有一个需求就是`Transform2D`的层级嵌套, 这算一个高级特性, 最后实现.
