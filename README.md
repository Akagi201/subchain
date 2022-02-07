# Subchain

## Build

```bash
cargo build --release
```

## Run dev chain

```bash
./target/release/subchain --dev --tmp
```

## Run subchain_testnet chain

```bash
./target/release/subchain --chain=subchain_testnet_raw.json
```

or

```bash
./target/release/subchain --chain=subchain_testnet
```

## 发布说明

在平行链上线之前，通过 benchmark 测出每个函数的性能消耗，可以更好的估算链上 gas 费用，
Chainspec 作为创世文件，配置了一条区块链创世区块的状态存储，如通证总量、议会初始成员等等，
而 runtime 的配置常常与特定 pallet 的业务逻辑相关，比如 staking 中的最小锁定额度，最大验证人数量以及锁定期等等。

以后大家在开发时注意区分，runtime 里的参数配置与 chainspec 的初始状态配置；
可供参考的方法是：如果某个参数一旦定义不再更改，则可以作为创世状态写在 chainspec 里（如 token 发行总量），未来可能会伴随着 runtime 升级更改，则配置在 runtime 里（如 staking 中的最小锁定额度）；和实际业务结合。

操作参考: <https://github.com/clubnetwork/exercise6>

