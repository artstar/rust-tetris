import * as wasm from "brick-game-wasm";
import {Action} from "brick-game-wasm";

function to2d(arr: Uint8Array, width: number): number[][] {
    let temp = Array.from(arr);
    let newArr = [];
    while (temp.length) {
        newArr.push(temp.splice(0, width));
    }
    return newArr;
}

function now(): BigInt {
    return BigInt(new Date().getTime())
}

class Playfield {
    constructor(
        public cols: number,
        public rows: number,
        public main: HTMLDivElement,
        public preview: HTMLDivElement,
        public score: HTMLDivElement,
        public menu: HTMLDivElement
    ) {
        let cells = new Array(cols * rows).fill('<div class="cell"></div>')
        this.main.innerHTML = cells.join("")
        let pcells = new Array(4 * 4).fill('<div class="cell"></div>')
        this.preview.innerHTML = pcells.join("")
    }

    render(main: number[], preview: number[], score: number) {
        this.textmode(false);
        this.draw(main, this.main);
        this.draw(preview, this.preview)
        this.score.innerText = score.toString();
    }

    private draw(cells: number[], block: HTMLDivElement) {
        let divs = Array.from(block.querySelectorAll(".cell"));
        for (let i = 0; i < divs.length; i++) {
            let list = divs[i].classList;
            if (!list.contains("color-" + (cells[i] || 0))) {
                list.remove(...Array.from(list).filter(t => t.startsWith("color")));
                list.add("color-" + (cells[i] || 0));
            }
        }
    }

    text(items: string[], selected?: number) {
        this.textmode(true);
        this.menu.innerHTML = items.map(item => `<div class="item">${item}</div>`).join("")
        if (selected !== undefined) {
            this.menu.querySelector(`.item:nth-child(${selected + 1})`).classList.add("selected")
        }
    }

    private textmode(enable: boolean) {
        if (enable) {
            this.menu.classList.add("visible");
        } else {
            this.menu.classList.remove("visible");
            this.menu.innerHTML = "";
        }
    }
}

let LastKey: Action = undefined;

document.addEventListener('keydown', ev => {
    switch (ev.code) {
        case "KeyW":
        case "ArrowUp":
            LastKey = Action.Up
            break;
        case "KeyA":
        case "ArrowLeft":
            LastKey = Action.Left
            break;
        case "KeyD":
        case "ArrowRight":
            LastKey = Action.Right
            break;
        case "KeyS":
        case "ArrowDown":
            LastKey = Action.Down
            break;
        case "Space":
        case "Enter":
            LastKey = Action.Drop
            break;
        case "Escape":
        case "Backspace":
            LastKey = Action.Escape
            break;
    }
})

let game = wasm.JSGame.start(now());
let renderer = new Playfield(
    20,
    10,
    document.querySelector<HTMLDivElement>(".playfield"),
    document.querySelector<HTMLDivElement>(".preview"),
    document.querySelector<HTMLDivElement>(".score"),
    document.querySelector<HTMLDivElement>(".menu")
)

function loop() {
    let state = game.tick(now(), LastKey);
    if (LastKey !== undefined) {
        LastKey = undefined;
    }
    switch (state.action) {
        case wasm.JSAction.Draw:
            renderer.render(Array.from(state.main()), Array.from(state.preview()), state.score());
            break;
        case wasm.JSAction.Text:
            renderer.text(state.text_items().split("\n"), state.text_selected())
            break;
        case wasm.JSAction.Exit:
            renderer.render([], [], 0);
            return;
    }
    window.requestAnimationFrame(loop);
}

window.requestAnimationFrame(loop);