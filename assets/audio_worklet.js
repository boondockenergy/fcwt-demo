
class WasmBridge extends AudioWorkletProcessor {
    /*
    constructor(options) {
        console.log("c");
        super();
        let [module, memory, handle] = options.processorOptions;
        bindgen.initSync(module, memory);
        this.processor = bindgen.WasmAudioProcessor.unpack(handle);
    }
    */
    process(inputs, outputs) {
        outputs[0][0].set(inputs[0][0]);
        outputs[0][1].set(inputs[0][0]);
        //return this.processor.process(outputs[0][0]);

        /*
        const output = outputs[0];
        output.forEach((channel) => {
        for (let i = 0; i < channel.length; i++) {
            channel[i] = Math.random() * 2 - 1;
        }
        });
        */

        return true
    }
}

console.log("WASM Audio Bridge")

registerProcessor("WasmProcessor", WasmBridge);