# The design of window module

## 先来讨论EventLoop产生的Input应该如何Send到主线程

---

Input分为Mouse和Keyboard(暂时); Mouse需要记录一些数据, 这些数据是固定的, EventLoop需要这些数据加上当前事件来计算出下一个Mouse; 因为玩家只需要读取状态, 而不涉及状态, 所以Mouse可以分为两个部分, MouseInner和Mouse, Mouse持有MouseInner的部分所有权(Arc), 把Mouse提供给用户, 因为Mouse只能读取; MouseInner供EventLoop进行持续维护, 这里有一个问题, 就是EventLoop和AppStage是并行的, 那么可能出现在同一个函数执行过程中, 因为读取位置不同而导致读取结果不同, 这是需要避免的!!!

思路变得稍微复杂一点, 在AppStage: Window中spawn了一个新线程用于执行EventLoop, Window维护一个Mouse, EventLoop维护一个Mouse, EventLoop当更新Mouse后会发送一个信号要求同步Mouse...

好麻烦啊, 要不直接把Event发送到主线程Window模块执行好了, 反正我只想要一个不会阻塞的EventLoop循环; 因为不想阻塞main thread, 所以在Window Moudle里面使用的是轮询, 轮询可能轮空, 也可能漏轮, 这非常危险, 因为想JUST_PRESSED这种状态只会出现一次;

也有解决思路, 因为winit的执行频率只有60fps, 只要把window的轮询频率调高就行了; 但感觉有些不美观!

我对winit的机制有些不熟悉, 里面应该有些我需要的方法!

第一个问题: 一个Mouse在何时应被认为修改完毕?

猜测: 应该在Event::RedrawEventCleared事件后.

共享状态可能轮空, 因为不知道变化了几次, 但channel可以确定每次的变化

可能出现的问题:

1. 帧率比window低的可能不能连续的接收Input

keyboard需要维护一张HashMap, 是键位与状态的对应关系表, 传递起来很困难, 用复制的方法内存不友好, 因为EventLoop和主线程间是并行关系, 可能出现同函数不同状态的情况, 所以解决方案还是用Window来维护Keyboard, EventLoop将事件传递到Window进行处理, 因为Event基本全是值类型!

TODO:

1. 了解winit的事件派发机制
2. 建立一个双线程模型, EventLoop单向发送Event到主线程

一些平台的窗口是需要劫持主线程的, 这就非常麻烦了, 换个思路好了! EventLoop在运行时放在主线程, 通过mpmc发送消息到次线程, 发送过程会有分拣;

这套路有点问题啊, window管理果然是个大活, 首先就是设计管道!!

让EventLoop劫持主线程, 将App放到副线程中运行!

现在问题是, 主线程被EventLoop劫持, 主线程有没有什么特殊的点会导致浪费一个核心的计算资源
