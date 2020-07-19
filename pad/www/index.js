import { GalaxyEvaluatorProxy } from "pad";

const CELL_SIZE = 5; // px
const canvas = document.getElementById("galaxy-canvas");
const proxy = GalaxyEvaluatorProxy.new()

proxy.interact();
console.log(proxy.debug());
