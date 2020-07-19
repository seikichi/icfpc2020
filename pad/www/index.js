import { GalaxyEvaluatorProxy } from "pad";

const canvas = document.getElementById("galaxy-canvas");
const ctx = canvas.getContext('2d');
let width = 0;
let height = 0;

const apiKey = (new URL(document.location)).searchParams.get('apiKey')
console.log(apiKey);
const CELL_SIZE = 20; // px
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

const interact = async (rows, cols) => {
    proxy.interact(rows, cols);

    while (proxy.needs_send()) {
        const body = proxy.get_send_body();
        const url = `/api/aliens/send?apiKey=${apiKey}`;
        const response = await fetch(url, { method: 'POST', body: body });
        const text = await response.text();
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
    const boundingRect = canvas.getBoundingClientRect();

    const scaleX = canvas.width / boundingRect.width;
    const scaleY = canvas.height / boundingRect.height;

    const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
    const canvasTop = (event.clientY - boundingRect.top) * scaleY;

    const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
    const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

    console.log(row, col);
    await update(row, col);
    drawGrid();
    drawCells();
});

(async () => {
    await update(0, 0);
    drawGrid();
    drawCells();
})();
