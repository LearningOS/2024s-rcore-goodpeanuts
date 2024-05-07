# chapter5练习

## 编程作业

### 进程创建

找了半天ch5的系统如何加载用户程序数据，后来发现都是从第一个测试程序中`spawn`出来的。

通过阅读源码发现`TaskControlBlock`中提供了`new`方法但是只有`initproc`使用了。阅读这个方法及其函数签名就会发现，我们只需要提供使用`get_app_data_by_name`解析出的`elf`程序数据即可创建新的`TaskControlBlock`。 

然后补充一下父子进程关系即可完成，`TaskControlBlock`的`new`方法提供给initproc的，没有设置父子关系，所以这里要设置一下。

### stride 调度算法

最低优先级为16, 因此选取`[7, 9, 11, 13, 15, 16]`的最小公倍数作为`BIG_STRIDE`。

`sys_set_priority`设置优先级时顺便会计算并设置任务的pass值。注意要求优先级必须大于等于2。

修改`fetch`方法，每次取出任务时，遍历一遍`ready_queue`，找到`stride`最小的任务将其值加上`pass`大小。然后用`Vec`的`remove`方法移出队列顺便返回。

## 问答作业

stride 算法深入

stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

实际情况是轮到 p1 执行吗？为什么？

- p2执行一个时间片后，它的stride会变成260。由于8位无符号整数的溢出。在这种情况下，如果一个进程的stride加上pass的值超过了8位无符号整数的最大值（255），那么它的stride将会溢出，实际上会变成4（260 - 256）。

我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， **在不考虑溢出的情况下** , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

为什么？尝试简单说明（不要求严格证明）。

+ 在stride调度算法中，每个进程的stride值是其优先级（或者说，其pass值）的累加。如果所有进程的优先级都大于等于2，那么每次调度时，stride的最大值和最小值之间的差距会受到限制。这是因为每次调度时，我们都选择stride最小的进程执行，并将其stride增加其优先级。因此，stride最小的进程的stride会增加，而其他进程的stride保持不变。这意味着，stride的最大值和最小值之间的差距最多增加优先级的最大值。如果所有进程的优先级都大于等于2，那么每次调度时，stride的最大值和最小值之间的差距最多增加2。因此，如果我们开始时所有进程的stride都相等（或者说，stride的最大值和最小值之间的差距为0），那么经过任意次调度后，stride的最大值和最小值之间的差距最多为2n，其中n是调度的次数。然而，由于我们每次都选择stride最小的进程执行，所以经过一定次数的调度后，所有进程的stride都会接近于其优先级的倍数。这意味着，stride的最大值和最小值之间的差距会趋向于0，而不是无限增大。因此，如果所有进程的优先级都大于等于2，那么在不考虑溢出的情况下，我们可以保证stride的最大值和最小值之间的差距不会超过BigStride / 2。这是因为BigStride / 2是优先级的最大可能值，而stride的最大值和最小值之间的差距不会超过优先级的最大值。
    
已知以上结论，**考虑溢出的情况下**，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 `partial_cmp` 函数，假设两个 Stride 永远不会相等。

TIPS: 使用 8 bits 存储 stride, BigStride = 255, 则: `(125 < 255) == false`, `(129 < 255) == true`.

```rust
 use core::cmp::Ordering;

 struct Stride(u64);

 impl PartialOrd for Stride {
     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
         // ...
         if (self.0 - other.0).abs > BigStride / 2 {
             if self.0 < other.0 {
                Some(Ordering::Less)
			 } else {
             	Some(Ordering::Greater)
             }
         } else {
             if self.0 > other.0 {
                Some(Ordering::Greater)
			} else {
             	Some(Ordering::Less)
             }
         }
     }
 }

 impl PartialEq for Stride {
     fn eq(&self, other: &Self) -> bool {
         false
     }
 }
```

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
    
2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

[rcore 实验三 马思源.mp4](https://cloud.tsinghua.edu.cn/d/eec08e3c8f224e27b01d/files/?p=%2Frcore%20%E5%AE%9E%E9%AA%8C%E4%B8%89%20%E9%A9%AC%E6%80%9D%E6%BA%90.mp4)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
    
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。