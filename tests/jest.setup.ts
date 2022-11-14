import console from "node:console";

function getTime() {
    const now = new Date(Date.now());

    const h = String(now.getHours()).padStart(2, "0");
    const m = String(now.getMinutes()).padStart(2, "0");
    const s = String(now.getSeconds()).padStart(2, "0");
    const ml = String(now.getMilliseconds()).padStart(3, "0");

    return `[${h}:${m}:${s}:${ml}]`;
}

function wrapLog(logger: (...args: string[]) => void) {
    return (...args: string[]) => {
        logger(getTime(), ...args);
    };
}

global.console = {
    ...console,
    log: wrapLog(console.log),
    error: wrapLog(console.error),
    debug: wrapLog(console.debug),
    info: wrapLog(console.info),
};
