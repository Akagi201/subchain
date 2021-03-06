# Kitties Pallet

## 题目

1. 增加买和卖的 extrinsic，对视频中 kitties 的实现进行重构，提取出公共代码

2. KittyIndex 不在 pallet 中指定，而是在 runtime 里面绑定

3. 测试代码能测试所有的五个方法，能检查所有定义的 event，能测试出所有定义的错误类型

4. 引入 Balances 里面的方法，在创建时质押一定数量的 token，在购买时支付 token

## cargo test

```bash
cargo test -p pallet-kitties -- --nocapture # make println! in test works
```

## 测试结果

![pallet_kitties_test](assets/pallet_kitties_test.png)

## 优秀作业

* 樊鹏飞: <https://github.com/xingfly/rust_pro_2/tree/master/pallets/kitties>
* 罗小龙: <https://github.com/yangluo1024/substrate-homeworks/tree/master/kitties-node/pallets/kitties>

## 参考资料

runtime 中绑定 kittyindex

* <https://github.com/paritytech/substrate/blob/monthly-2021-10/frame/balances/src/lib.rs#L233>
* <https://github.com/substrate-developer-hub/substrate-node-template/blob/v3.0.0%2Bmonthly-2021-10/runtime/src/lib.rs#L246>

检查 event

* <https://github.com/paritytech/substrate/blob/monthly-2021-10/frame/balances/src/tests.rs#L728-L755>

检查错误类型

* <https://github.com/paritytech/substrate/blob/monthly-2021-10/frame/balances/src/tests.rs#L148-L166>

质押

* <https://github.com/paritytech/substrate/blob/monthly-2021-10/frame/proxy/src/lib.rs#L123>
* <https://github.com/paritytech/substrate/blob/monthly-2021-10/frame/proxy/src/lib.rs#L327>
* <https://github.com/paritytech/substrate/blob/monthly-2021-10/bin/node/runtime/src/lib.rs#L319>

Rust 关联类型

* <https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
