<!--lint disable no-literal-urls-->
<div align="center">
  <h1>Rust AV1 Decoder</h1>
</div>
<br/>
<div align="center">
  <strong>A AV1 decoder implemented in pure rust.</strong>
</div>
<div align="center">
  <img src="https://img.shields.io/github/license/mycrl/toy-rav1d"/>
  <img src="https://img.shields.io/github/issues/mycrl/toy-rav1d"/>
  <img src="https://img.shields.io/github/stars/mycrl/toy-rav1d"/>
</div>
<div align="center">
  <sup>This is an experimental project currently in progress.</sup>
</div>

***

Unlike existing projects, this is an AV1 decoder implemented entirely from scratch in Rust. However, please note that this is just a side project and not intended for production use, so there is no specific focus on performance, and not all features are supported.

***


## Roadmap

#### Obu

* [x] sequence header.
* [x] metadata.
* [ ] frame header.
* [ ] frame.
* [x] tile list.
* [ ] tile group.
* [ ] padding.
* [ ] redundant frame header.
* [ ] temporal delimiter.

#### Reconstruction

* [ ] DCT.
* [ ] ADST.
* [ ] walsh hadamard.
* [ ] CDEF.
* [ ] loop filter.
* [ ] motion field estimation.
* [ ] motion vector prediction.
* [ ] intra prediction.
* [ ] inter prediction.
* [ ] box filter.
* [ ] self guided filter.
* [ ] wiener filter.

#### Public Interface

* [ ] av1 decoder.
* [ ] av1 decoder error.
* [ ] obu enum.

#### Other

* [ ] tests.
* [ ] examples.


### License
[GPL](./LICENSE) Copyright (c) 2023 Mr.Panda.
