import { GalaxyEvaluatorProxy, format_modulated_string } from "pad";

const canvas = document.getElementById("galaxy-canvas");

const status_elem = document.getElementById("status");
const current_url_elem = document.getElementById("current-url");
const current_state_elem = document.getElementById("current-state");
const previous_state_elem = document.getElementById("previous-state");
const previous_vector_elem = document.getElementById("previous-vector");
const previous_modulated_state_elem = document.getElementById("previous-state-modulated");
const previous_modulated_vector_elem = document.getElementById("previous-vector-modulated");

const last_send_input_elem = document.getElementById("last-send-input");
const last_send_output_elem = document.getElementById("last-send-output");
const last_send_modulated_input_elem = document.getElementById("last-send-input-modulated");
const last_send_modulated_output_elem = document.getElementById("last-send-output-modulated");

const ctx = canvas.getContext('2d');
let width = 0;
let height = 0;
let last_send_input = "";
let last_send_output = "";
let last_send_modulated_input = "";
let last_send_modulated_output = "";

const url = new URL(document.location);
const searchParams = url.searchParams;
const origin = url.origin;
const apiKey = searchParams.get('apiKey');

if (!apiKey) {
    alert("INVALID URL: missing apiKey query parameter");
}

const initial_state = searchParams.get('state');
const initial_vector = searchParams.get('vector');

const CELL_SIZE = 5; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

const colors = [
    "rgba(128,   0, 0, 0.5)",
    "rgba(  0, 128, 0, 0.5)",
    "rgba(  0,   0, 128, 0.5)",
    "rgba(255,   0, 0, 0.5)",
    "rgba(  0, 255, 0, 0.5)",
    "rgba(  0,   0, 255, 0.5)",
    "rgba(128, 128, 0, 0.5)",
    "rgba(  0, 128, 128, 0.5)",
    "rgba(128,   0, 128, 0.5)",
];

const proxy = GalaxyEvaluatorProxy.new()

if (initial_state && initial_vector) {
    proxy.restore(initial_state, initial_vector);
}

let processing = false;
const disableCanvas = () => {
    processing = true;
    canvas.style = 'pointer-events: none;';
    status_elem.textContent = "processing";
}
const enableCanvas = () => {
    processing = false;
    canvas.style = '';
    status_elem.textContent = "done";
};

const updateInfo = () => {
    current_url_elem.href = `${origin}?apiKey=${apiKey}&state=${proxy.get_previous_state()}&vector=${proxy.get_previous_vector()}`;
    current_state_elem.textContent = proxy.get_current_state_for_human();
    previous_state_elem.textContent = proxy.get_previous_state_for_human();
    previous_vector_elem.textContent = proxy.get_previous_vector_for_human();
    previous_modulated_state_elem.textContent = proxy.get_previous_state();
    previous_modulated_vector_elem.textContent = proxy.get_previous_vector();

    last_send_input_elem.textContent = last_send_input;
    last_send_output_elem.textContent = last_send_output;
    last_send_modulated_input_elem.textContent = last_send_modulated_input;
    last_send_modulated_output_elem.textContent = last_send_modulated_output;
};

const interact = async (rows, cols) => {
    proxy.start_interaction(rows, cols);

    while (proxy.needs_send()) {
        const body = proxy.get_send_body();
        last_send_modulated_input = body;
        last_send_input = format_modulated_string(body);
        const url = `/api/aliens/send?apiKey=${apiKey}`;
        const response = await fetch(url, { method: 'POST', body: body });
        const text = await response.text();
        last_send_modulated_output = text;
        last_send_output = format_modulated_string(text);

        proxy.continue_interaction(text);
    }
};

const update = async (row, col) => {
    await interact(row, col);
    console.log(proxy.debug());

    width = proxy.width();
    height = proxy.height();

    canvas.height = (CELL_SIZE + 1) * height + 1;
    canvas.width = (CELL_SIZE + 1) * width + 1;
};

const drawGrid = () => {
    ctx.beginPath();
    ctx.strokeStyle = GRID_COLOR;

    // Vertical lines.
    for (let i = 0; i <= width; i++) {
        ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
        ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
    }

    // Horizontal lines.
    for (let j = 0; j <= height; j++) {
        ctx.moveTo(0, j * (CELL_SIZE + 1) + 1);
        ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
    }

    ctx.stroke();
};

const drawCells = () => {
    ctx.globalAlpha = 0.5;
    ctx.globalCompositeOperation = "lighter";

    ctx.beginPath();

    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            const idx = proxy.color(row, col);

            ctx.fillStyle = DEAD_COLOR;
            ctx.fillRect(
                col * (CELL_SIZE + 1) + 1,
                row * (CELL_SIZE + 1) + 1,
                CELL_SIZE,
                CELL_SIZE
            );

            for (let i = 0; i < 32; i++) {
                let b = idx & (1 << i)
                if (b === 0) { continue; }

                ctx.fillStyle = colors[i];

                ctx.fillRect(
                    col * (CELL_SIZE + 1) + 1,
                    row * (CELL_SIZE + 1) + 1,
                    CELL_SIZE,
                    CELL_SIZE
                );
            }
        }
    }

    ctx.stroke();
};

canvas.addEventListener("click", async event => {
    if (processing) {
        return;
    }

    const boundingRect = canvas.getBoundingClientRect();

    const scaleX = canvas.width / boundingRect.width;
    const scaleY = canvas.height / boundingRect.height;

    const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
    const canvasTop = (event.clientY - boundingRect.top) * scaleY;

    const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
    const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

    disableCanvas();
    await update(row, col);
    drawGrid();
    drawCells();
    enableCanvas();
    updateInfo();
});

(async () => {
    disableCanvas();
    await update(0, 0);
    drawGrid();
    drawCells();
    enableCanvas();
    updateInfo();
})();
