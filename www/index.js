import * as wasm from "hello-wasm-pack";

wasm.setup();

const render = () => {
    wasm.render();
    requestAnimationFrame(render)
}
render()