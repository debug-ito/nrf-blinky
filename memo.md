
# 2020-03-01

API設計について考察。

HALにならうのがいい気がする。
singleton moduleでデバイスオブジェクトをconsumeして、引き換えにagent objectを返す。
そうすればデバイスオブジェクトがstatic Mutex内に入っていることをruntimeで保証できる。



# 2020-02-29

GPIOTEモジュールを読んでみた。
ピン出力をTASKSでできるらしい。トグルもできる。

GPIOTEは8チャネルある。
CONFIGでチャネルを設定。
各チャネルは独立にport/pinに割り当てられる。
CONFIG.POLARITYがレベル設定。
TaskモードのときはOUTタスクが来たときに実行するアクションを指定。
Eventモードのときは検出エッジを指定する。
SETタスク、CLRタスクはCONFIGの設定によらずpinをhigh/lowにセットする。

なるほど、かなり分かりやすいし、こっちのほうが使いやすそう。


どういうAPIにするか？

GPIOTEオブジェクトをsingletonとしてstatic mutに持っていく。

IN設定とOUT設定、どっちかをチャネルを指定してセットすればいい。
IN設定はコールバック関数を渡せるようにする。
OUT設定は、agentオブジェクトを返すようにするか？
それでもいいけど、途中で設定を変える場合は面倒。
それはどのみち難しいから、考えなくていいかなあ。

とりあえず入力割り込みだけ試せればいいか。



# 2020-02-26

`assert_blink`関数を作ってちょっと内部状態を調べてみた。

- PRIMASKは`interrupt::enable`するまでもなくActiveになっている。(つまりマスクされていない。ゼロ。)
- VTORは0になってる！

VTORをちゃんとした値にセットしたら動いた。。 やれやれ。

`__reset_vector`シンボルからベクターテーブルのアドレスをぶっこ抜いてVTORにセットした。これでOK。

しかしこれ、なんでVTORがセットされてないんだろう。
多分、ブートローダーがセットしていないのだと思う。
`cortex-m-rt`のスタートアップコードを見たが、特にVTORをリセットしているコードは見当たらなかった。

だとすると、nRF5 SDKのコードでうまく動いたのは、あれがユーザーコードでVTORをセットしていたからか？
nRF5 SDKのコードをvtorでackしてみたが、特にVTORをいじっているようには見えない。
example timerのビルドでコンパイルしているコードのうち怪しいものをざっと見たが、VTORをセットしているように見えない。

よく分からんが、、まあ、いいか。


# 2020-02-25

http://kevincuzner.com/2018/11/13/bootloader-for-arm-cortex-m0-no-vtor/

Cortex-MのVTOR(Vector Table Offset Register)とブートローダについての記事。

あれ、もしかしてブートローダーが割り込みをグローバルに無効化している？？

と思ったけど、`cm_interrupt::enable()`呼んでもダメだった。
マジ？
PRIMASKの状態は`cortex_m::register`以下で読めそう。
あとVTORの値も気になる。
VTORをちゃんと書き換えればうまくいかないかなあ。



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
セットしてみたけど、効果なし。

Timer割り込みハンドラから直接LEDを触ろうともしたが、toggleさせるのが妙にめんどくさかったらいったん諦めた。

Cargo.tomlではcortex-mにdevice featureを、PACにrt featureをいれているが、device featureを抜いてみた。
効果なし。

Timer0 interrupt handlerから直接LEDを操作してみたが、効果なし。
どうやらそもそもhandlerが呼ばれていないような感じ。

vector tableはちゃんとセットされているのか？

https://developer.arm.com/docs/dui0553/latest/the-cortex-m4-processor/exception-model/vector-table

Cortex-M4では、

- 0x0000: initial SP
- 0x0004: Reset
- [0x0008, 0x0040): Exceptions
- [0x0040, ):  IRQs

て感じ。
各エントリは4バイトで、飛び先アドレスが書かれているはず。

で、Timer0のperipheral IDは8なので、0x0040 + 4 * 8 = 0x0060 がTimer0のIRQ entryのはず。

なお、今回は0x026000をFLASHアドレスの先頭なので、そこでオフセットされている。


objdumpでvector tableを見ると、何かしら値が書かれてはいる。これは正しいのか？
あ、endianが見た目と逆なのかな。

それでいうと、ベクタエントリはアドレス00027543, TIMER0のアドレスは00027542。
LSBはなんか別の意味があるんだっけ？
だったらこれは正しそうな気がするんだが。
でもTIMER0の中には大した内容がないような・・・？

うーん、でもcortex-m-rtやnrf52840-pacの内容を見る限り、問題ないように思うんだけど。

nRF5 SDKのexamples/peripheral/timerをマニュアル通りにSparkFunビルドに移植してブートローダーで書き込んだら一応Lチカはできてる。
でもLチカの周期がなんかおかしいような。
点灯時間と消灯時間がアンバランス。
objdumpで中身を見たが、vector entry at 0x26060 = 0002652d で、0002652c <TIMER0_IRQHandler>.
ちゃんと整合している気がする。

nrf52-hal-commonのtimer実装を見たけど、`enable_interrupt`ではintensetをONにしてnvic.enableしている。

nrf52840-halはデフォルトで"rt" featureが入っている気がする。
で、-pacにも"rt" featureを入れる。
するとcortex-m-rtには"device" featureが入るようになっている、はず。


マジでさっぱり分からん。
SysTick exceptionを使えないか試してみるか？
NVICは関係ないけど。

Cortex-Mの System Control Block (SCB)とかは？
Vector Table Offset Registerとかあるんだけど？
これ、SDKではファームウェアでセットしたりしてるのか？


参考

- https://github.com/rust-embedded/cortex-m/issues/154
  - https://github.com/geomatsi/rust-blue-pill-tests/blob/master/src/bin/blink-timer-irq-safe.rs
