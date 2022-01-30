# template

## Generate weights.ts from template

```bash
cargo build --features runtime-benchmarks --release
```

以下结果使用我的 2018 年 macbook pro 测试结果，如果按照标准配置会比这个时间长一些。


```bash
❯ ./target/release/subchain benchmark --chain=dev --steps='50' --repeat='20' --pallet=pallet_template --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages='4096' --output='./pallets/template/src/weights.rs' --template='./.maintain/frame-weight-template.hbs'
Pallet: "pallet_template", Extrinsic: "do_something", Lowest values: [], Highest values: [], Steps: 50, Repeat: 20
Raw Storage Info
========
Storage: TemplateModule Something (r:0 w:1)

Median Slopes Analysis
========
-- Extrinsic Time --

Model:
Time ~=       19
    + s        0
              µs

Reads = 0 + (0 * s)
Writes = 1 + (0 * s)

Min Squares Analysis
========
-- Extrinsic Time --

Data points distribution:
    s   mean µs  sigma µs       %
    0        19         0    0.0%
   20      19.6     0.489    2.4%
   40      18.5       0.5    2.7%
   60      19.5       0.5    2.5%
   80      19.2       0.4    2.0%
  100      19.9       0.3    1.5%
  120        20         0    0.0%
  140      18.1       0.3    1.6%
  160      18.7     0.458    2.4%
  180      18.5       0.5    2.7%
  200      19.2       0.4    2.0%
  220      18.4     0.489    2.6%
  240        18         0    0.0%
  260      18.2       0.4    2.1%
  280      18.1       0.3    1.6%
  300      18.5       0.5    2.7%
  320      17.5       0.5    2.8%
  340        18         0    0.0%
  360      19.4     0.489    2.5%
  380      19.8       0.6    3.0%
  400      18.5       0.5    2.7%
  420      17.5       0.5    2.8%
  440      18.6     0.489    2.6%
  460      18.2       0.4    2.1%
  480      17.9       0.3    1.6%
  500      19.1       0.3    1.5%
  520      18.3     0.458    2.5%
  540        18         0    0.0%
  560        18         0    0.0%
  580      18.5       0.5    2.7%
  600      18.9       0.3    1.5%
  620        20         0    0.0%
  640      18.9       0.3    1.5%
  660      19.5     0.806    4.1%
  680      18.6     0.489    2.6%
  700      18.7     0.458    2.4%
  720      17.6     0.489    2.7%
  740      18.6     0.489    2.6%
  760        18         0    0.0%
  780      18.7     0.458    2.4%
  800        19         0    0.0%
  820      18.3     0.458    2.5%
  840        18         0    0.0%
  860        18         0    0.0%
  880      19.7     0.458    2.3%
  900      19.5       0.5    2.5%
  920      18.5       0.5    2.7%
  940      18.1       0.3    1.6%
  960      18.4     0.489    2.6%
  980      18.5       0.5    2.7%
 1000      18.2       0.4    2.1%

Quality and confidence:
param     error
s             0

Model:
Time ~=    18.85
    + s        0
              µs

Reads = 0 + (0 * s)
Writes = 1 + (0 * s)
```
