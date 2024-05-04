# chapter4练习

## 重写 `sys_get_time`

结合ch3中的`sys_get_time`实现并且参考实验指导书上的`sys_write`实现方式，使用`translated_byte_buffer`读取内存中的数据，获取指向目标位置的一段连续内存`buffer`。不同于`sys_write`，这里需要将获取的`buffer`作为可变引用同时转换成`TimeVal`类型的指针，然后在unsafe块中解引用指针进行赋值。

## 重写 `sys_task_info`

和上面一样使用`translated_byte_buffer`获取内存中的数据内存中的数据，将指针转成`TaskInfo`类型，然后解引用赋值。


## 

 virt_end = VirtAddr::from((_start + _len + 4095) / 4096 * 4096) 对齐

from_bits是bitflags库提供的一个方法，用于从一个原始的位模式创建一个位标志集合。这个方法会检查所有的位，如果有任何无效的位被设置，它会返回None。

在你给出的代码中，MapPermission是一个位标志集合，它定义了四个权限：R（可读）、W（可写）、X（可执行）和U（用户模式下可访问）。你可以使用from_bits方法从一个u8值创建一个MapPermission，例如：

```rust
let bits: u8 = ...; // your u8 value

if let Some(permission) = MapPermission::from_bits(bits) {
    // permission is a valid MapPermission
} else {
    // bits contains some invalid bits
}
```

mmap时需要判断是否合法，否则会把之前的测试样例崩掉

```rust
if inner.tasks[cur].memory_set.translate(VirtPageNum::from(start_va)).is_some() || inner.tasks[cur].memory_set.translate(VirtPageNum::from(end_va)).is_some(){
            return;
        }
```