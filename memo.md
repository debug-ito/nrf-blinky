
# 2020-02-24

interrupt handlerをセットするのが意外とややこしい。

- cortex-m-rt crateのfeature = "device"を有効に
  https://docs.rs/cortex-m-rt/0.6.12/cortex_m_rt/#optional-features
- PACのfeature = "rt"を有効に
  https://docs.rs/svd2rust/0.17.0/svd2rust/#interrupt-api

しないといけないっぽい。
これでうまく行くかどうかは未検証。


タイマー割り込みが使えない。
おそらくだが、割り込みハンドラ自体が呼ばれていないように思える。

nRF SDKの examples/peripheral/timer を見てみるが、これも随分ハイレベルなSDKを使っている。
とはいえ、裏側でinterruptを使っている模様。

初期化ルーチンは modules/nrfx/drivers/src/nrfx_timer.c の `nrfx_timer_init`関数。
ここでNVICの割り込みを有効化している。priorityもセットしているようだ。

で、その後`nrfx_timer_extended_compare`関数でcompare eventをクリアしてからtimer側のinterruptを有効化している。多分。

priorityをセットしないといけないのか？


Timer割り込みハンドラから直接LEDを触ろうともしたが、toggleさせるのが妙にめんどくさかったらいったん諦めた。


参考

- https://github.com/rust-embedded/cortex-m/issues/154
  - https://github.com/geomatsi/rust-blue-pill-tests/blob/master/src/bin/blink-timer-irq-safe.rs
