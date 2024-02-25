use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, MediaStreamConstraints, MediaStream};

pub struct AudioWorker {

}

#[wasm_bindgen]
pub struct WasmAudioProcessor(Box<dyn FnMut(&mut [f32]) -> bool>);

#[wasm_bindgen]
impl WasmAudioProcessor {
    pub fn process(&mut self, buf: &mut [f32]) -> bool {
        log::info!("Sink process {}", buf.len());
        self.0(buf)
    }
    pub fn pack(self) -> usize {
        log::info!("pack");
        Box::into_raw(Box::new(self)) as usize
    }
    pub unsafe fn unpack(val: usize) -> Self {
        log::info!("unpack");
        *Box::from_raw(val as *mut _)
    }
}

struct Sink {
}

impl Sink {
    pub fn process(&mut self, buf: &mut [f32]) -> bool {
        log::info!("Sink process {}", buf.len());
        true
    }
}

pub fn audio_init() -> AudioWorker {
    wasm_bindgen_futures::spawn_local(async {
        let ctx = AudioContext::new().unwrap();

        JsFuture::from(ctx.audio_worklet().unwrap().add_module("audio_worklet.js").unwrap()).await.unwrap();
        JsFuture::from(ctx.audio_worklet().unwrap().add_module("noise_worklet.js").unwrap()).await.unwrap();

        let window = web_sys::window().unwrap();
        let navigator = window.navigator();
        let media = navigator.media_devices().unwrap();

        let mut constraint = MediaStreamConstraints::new();
        constraint.audio(&JsValue::from(true));
        let stream: MediaStream = JsFuture::from(media.get_user_media_with_constraints(&constraint).unwrap()).await.unwrap().into();

        let source = ctx.create_media_stream_source(&stream).unwrap();

        //let sink_node = wasm_audio_node(&ctx, Box::new(move |buf| sink.process(buf) )).unwrap();
        let sink_node = AudioWorkletNode::new(&ctx, "WasmProcessor").unwrap();
        let noise_node = AudioWorkletNode::new(&ctx, "random-noise-processor").unwrap();

        source.connect_with_audio_node(&sink_node).unwrap();
        //source.connect_with_audio_node(&sink_node).unwrap();
        sink_node.connect_with_audio_node(&ctx.destination()).unwrap();

        let ch = sink_node.channel_count();

        log::info!("Inputs: {} Outputs: {} Channels: {}", sink_node.number_of_inputs(), sink_node.number_of_outputs(), ch);

        log::info!("Audio worker initialized");
    });

    AudioWorker{}
}

pub fn wasm_audio_node(
    ctx: &AudioContext,
    process: Box<dyn FnMut(&mut [f32]) -> bool>,
) -> Result<AudioWorkletNode, JsValue> {

    AudioWorkletNode::new_with_options(
        ctx,
        "WasmProcessor",
        AudioWorkletNodeOptions::new().processor_options(Some(&js_sys::Array::of3(
            &wasm_bindgen::module(),
            &wasm_bindgen::memory(),
            &WasmAudioProcessor(process).pack().into(),
        ))),
    )
}

/*
pub async fn prepare_wasm_audio(ctx: &AudioContext) -> Result<(), JsValue> {
    let mod_url = crate::dependent_module!("worklet.js")?;
    JsFuture::from(ctx.audio_worklet()?.add_module(&mod_url)?).await.unwrap();
    Ok(())
}
*/