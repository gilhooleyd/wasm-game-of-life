import { Universe } from "./pkg/wasm.js";
import init from './pkg/wasm.js';


function run() {
  const pre = document.getElementById("game-of-life-canvas");
  const universe = Universe.new();
  const renderLoop = () => {
    pre.textContent = universe.render();
    universe.tick();

    requestAnimationFrame(renderLoop);
  };
  requestAnimationFrame(renderLoop);
}

init().then(run)
