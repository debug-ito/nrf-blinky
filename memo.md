
# 2020-02-24

interrupt handlerをセットするのが意外とややこしい。

- cortex-m-rt crateのfeature = "device"を有効に
  https://docs.rs/cortex-m-rt/0.6.12/cortex_m_rt/#optional-features
- PACのfeature = "rt"を有効に
  https://docs.rs/svd2rust/0.17.0/svd2rust/#interrupt-api

しないといけないっぽい。
これでうまく行くかどうかは未検証。

