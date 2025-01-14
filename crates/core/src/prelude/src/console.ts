
declare global {
    interface Console {
        debug(...data: any[]): void;
        error(...data: any[]): void;
        info(...data: any[]): void;
        log(...data: any[]): void;
        warn(...data: any[]): void;
    }

    /**
     * @internal
     */
    var __consoleWrite: (level: string, message: string) => void;
}

function stringifyArg(arg: any): string {
    if (arg === null) return 'null';
    if (arg === undefined) return 'undefined';
    if (typeof arg === 'object') {
        if (Array.isArray(arg)) {
            return arg.map(stringifyArg).join(' ');
        }
        try {
            return JSON.stringify(arg);
        } catch {
            return Object.prototype.toString.call(arg);
        }
    }
    return String(arg);
}

function createLogFunction(level: string) {
    return function (...args: any[]) {
        const message = args.map(stringifyArg).join(' ');
        __consoleWrite(level, message);
    };
}

const console = {
    log: createLogFunction('info'),
    info: createLogFunction('info'),
    warn: createLogFunction('warn'),
    error: createLogFunction('error'),
    debug: createLogFunction('debug')
};

globalThis.console = console;

export { };