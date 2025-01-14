
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
    
    if (typeof arg === 'symbol') return arg.toString();
    if (typeof arg === 'bigint') return `${arg}n`;
    if (typeof arg === 'function') return `[Function ${arg.name ? `${arg.name}` : '(anonymous)'}]`;

    if (typeof arg === 'object') {
        if (arg instanceof Error) {
            return arg.stack || `${arg.name}: ${arg.message}`;
        }
        if (arg instanceof Set) {
            return `Set(${arg.size}) { ${Array.from(arg).map(String).join(', ')} }`;
        }
        if (arg instanceof Map) {
            return `Map(${arg.size}) { ${Array.from(arg).map(([k, v]) => `${k} => ${v}`).join(', ')} }`;
        }
        if (Array.isArray(arg)) {
            const items = [];
            for (let i = 0; i < arg.length; i++) {
                items.push(i in arg ? stringifyArg(arg[i]) : '<empty>');
            }
            return `[ ${items.join(', ')} ]`;
        }

        // For regular objects, use JSON.stringify first for clean output
        try {
            return JSON.stringify(arg);
        } catch {
            // For objects that can't be JSON stringified (circular refs etc)
            // fall back to Object.prototype.toString behavior
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