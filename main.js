import init, { Cell, Universe } from './pkg/game_of_life.js'


/// Initialize UI associated with this app. Returns:
/// {
///   canvas: A canvas.mainDisplay,
///   controls: {
///     addButton: (title, clickAction),
///     elem: HTMLElement
///   }
/// }
function initUI(parent) {
    const container = document.createElement("div");
    container.classList.add("mainRegion");

    const controls = document.createElement("div");
    controls.classList.add("controls");
    container.appendChild(controls);

    const canvas = document.createElement("canvas");
    canvas.classList.add("mainDisplay");
    container.appendChild(canvas);

    parent.appendChild(container);

    return {
        canvas,
        controls: {
            addButton: (title, clickAction) => {
                const btn = document.createElement("button");
                btn.textContent = title;
                btn.addEventListener("keyup", event => {
                    if (event.key == "Enter") clickAction(event);
                });
                btn.addEventListener("click", clickAction);

                controls.appendChild(btn);
                return btn;
            },
            addInput: (label, type, onChange, configure) => {
                const labelElem = document.createElement("label");
                labelElem.textContent = label;

                const input = document.createElement("input");
                input.type = type;
                input.setAttribute("placeholder", label);

                labelElem.appendChild(input);
                controls.appendChild(labelElem);

                input.addEventListener("input", (evt) => {
                    onChange(input.value, evt);
                });

                if (configure) configure(input);

                return input;
            },
            element: controls,
        },
    };
}

async function run() {
    let running = true;
    let updateBtnText, updateSquareSize, mainloop, togglePaused, clearUniverse;
    let playPauseButton;
    let render;
    let updateRate = 1;

	await init();

    let universe = Universe.new(64, 64);
    let uiData = initUI(document.body);
    const canvas = uiData.canvas;
    const controls = uiData.controls;

    playPauseButton = controls.addButton("Pause", () => togglePaused());
    controls.addButton("Clear", () => clearUniverse());

    controls.addInput("Width: ", "number", (value) => {
        if (Math.floor(value) == value && value > 0 && !isNaN(value)) {
            universe.resize_to(value, universe.height());
            updateSquareSize();

            render();
        }
    }, (input) => {
        input.value = universe.width();
    });

    controls.addInput("Height: ", "number", (value) => {
        if (Math.floor(value) == value && value > 0 && !isNaN(value)) {
            universe.resize_to(universe.width(), value);
            updateSquareSize();

            render();
        }
    }, (input) => {
        input.value = universe.height();
    });

    controls.addInput("Rate: ", "range", (value) => {
        updateRate = value;
    }, (input) => {
        input.value = updateRate;
        input.min = 1;
        input.max = 12;
    });

    updateBtnText = () => {
        playPauseButton.textContent = running ? "Pause" : "Play";
    };

    updateSquareSize = () => {
        let square_size = Math.min(canvas.width, canvas.height)/Math.max(universe.width(), universe.height()) - universe.get_square_spacing();
        universe.set_square_size(square_size);
    };

    render = () => {
        if (canvas.width != canvas.clientWidth || canvas.height != canvas.clientHeight) {
            canvas.width = canvas.clientWidth;
            canvas.height = canvas.clientHeight;

            updateSquareSize();
        }


        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.fillStyle = "white";
        universe.fill_cells(Cell.Dead, ctx);

        ctx.fillStyle = "black";
        universe.fill_cells(Cell.Alive, ctx);
    };

    clearUniverse = () => {
        universe.clear();
        render();
    };

    togglePaused = () => {
        running = !running;
        updateBtnText();

        if (running) {
            mainloop();
        } else {
            render();
        }
    };

    let lastCellX, lastCellY;
    const handlePtrEvent = evt => {
        const bbox = canvas.getBoundingClientRect();
        const squareSize = universe.get_square_size() + universe.get_square_spacing();

        const x = Math.floor((evt.clientX - bbox.left) / squareSize);
        const y = Math.floor((evt.clientY - bbox.top) / squareSize);

        if (x == lastCellX && y == lastCellY) {
            return;
        } else if (x >= universe.width() || y >= universe.height()) {
            return;
        }

        universe.toggle_cell_at(x, y);

        if (lastCellX !== undefined && lastCellY !== undefined) {
            universe.toggle_cells_between(x, y, lastCellX, lastCellY);
        }

        lastCellX = x;
        lastCellY = y;
        render();
    };

    let ptrDown = false;
    canvas.addEventListener("pointerdown", (evt) => {
        if (evt.isPrimary) {
            ptrDown = true;
            lastCellX = undefined;
            lastCellY = undefined;
            evt.preventDefault();

            handlePtrEvent(evt);
        } else {
            ptrDown = false;
        }
    });
    canvas.addEventListener("pointerup", (evt) => ptrDown = false);
    canvas.addEventListener("pointermove", (evt) => {
        if (ptrDown) {
            evt.preventDefault();
            handlePtrEvent(evt);
        }
    });

    document.body.addEventListener("keydown", evt => {
        if (evt.key == "p") {
            togglePaused();
        }
    });

    const ctx = canvas.getContext("2d");

    mainloop = async function() {
        while (running) {
            for (let i = 0; i < updateRate; i++) {
                universe.tick();
            }

            render();
            await (new Promise((resolve, reject) => requestAnimationFrame(resolve)));
        }
    }

    await mainloop();
}

run();