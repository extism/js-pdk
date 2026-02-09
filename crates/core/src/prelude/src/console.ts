
declare global {
    interface Console {
        assert(condition?: boolean, ...data: any[]): void;
        count(label?: string): void;
        countReset(label?: string): void;
        debug(...data: any[]): void;
        error(...data: any[]): void;
        info(...data: any[]): void;
        log(...data: any[]): void;
        time(label?: string): void;
        timeEnd(label?: string): void;
        timeLog(label?: string, ...data: any[]): void;
        trace(...data: any[]): void;
        warn(...data: any[]): void;
    }

    /**
     * @internal
     */
    var __consoleWrite: (level: string, message: string) => void;

    /**
     * @internal
     */
    function __getTimeMs(): number;
}

function stringifyArg(arg: any): string {
    if (arg === null) return 'null';
    if (arg === undefined) return 'undefined';
    
    if (typeof arg === 'symbol') return arg.toString();
    if (typeof arg === 'bigint') return `${arg}n`;
    if (typeof arg === 'function') return `[Function ${arg.name ? `${arg.name}` : '(anonymous)'}]`;

    if (typeof arg === 'object') {
        if (arg instanceof Error) {
            return `${arg.name}: ${arg.message}${arg.stack ? '\n' : ''}${arg.stack}`;
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

const timers: Record<string, number> = {};
const counters: Record<string, number> = {};

const console = {
    trace: createLogFunction('trace'),
    debug: createLogFunction('debug'),
    log: createLogFunction('info'),
    info: createLogFunction('info'),
    warn: createLogFunction('warn'),
    error: createLogFunction('error'),

    assert(condition?: boolean, ...data: any[]) {
        if (!condition) {
            const message = data.length > 0
                ? `Assertion failed: ${data.map(stringifyArg).join(' ')}`
                : 'Assertion failed';
            __consoleWrite('error', message);
        }
    },

    time(label: string = 'default') {
        if (label in timers) {
            __consoleWrite('warn', `Timer '${label}' already exists`);
            return;
        }
        timers[label] = __getTimeMs();
    },

    timeEnd(label: string = 'default') {
        if (!(label in timers)) {
            __consoleWrite('warn', `Timer '${label}' does not exist`);
            return;
        }
        const elapsed = __getTimeMs() - timers[label];
        delete timers[label];
        __consoleWrite('info', `${label}: ${elapsed}ms`);
    },

    timeLog(label: string = 'default', ...data: any[]) {
        if (!(label in timers)) {
            __consoleWrite('warn', `Timer '${label}' does not exist`);
            return;
        }
        const elapsed = __getTimeMs() - timers[label];
        const extra = data.length > 0 ? ` ${data.map(stringifyArg).join(' ')}` : '';
        __consoleWrite('info', `${label}: ${elapsed}ms${extra}`);
    },

    count(label: string = 'default') {
        counters[label] = (counters[label] || 0) + 1;
        __consoleWrite('info', `${label}: ${counters[label]}`);
    },

    countReset(label: string = 'default') {
        if (!(label in counters)) {
            __consoleWrite('warn', `Count for '${label}' does not exist`);
            return;
        }
        counters[label] = 0;
    },

    table(data: any, columns?: string[]) {
        if (data === null || data === undefined || typeof data !== 'object') {
            __consoleWrite('info', stringifyArg(data));
            return;
        }

        let rows: any[];
        let indices: string[];

        if (Array.isArray(data)) {
            if (data.length === 0) {
                __consoleWrite('info', '(empty array)');
                return;
            }
            indices = data.map((_: any, i: number) => String(i));
            rows = data;
        } else {
            indices = Object.keys(data);
            rows = indices.map(k => data[k]);
        }

        const keySet: Record<string, boolean> = {};
        for (let i = 0; i < rows.length; i++) {
            const row = rows[i];
            if (row !== null && row !== undefined && typeof row === 'object') {
                Object.keys(row).forEach(k => { keySet[k] = true; });
            }
        }
        let keys = Object.keys(keySet);

        if (columns) {
            keys = keys.filter(k => columns.indexOf(k) !== -1);
        }

        if (keys.length === 0) {
            const lines = ['(index)\tValues'];
            for (let i = 0; i < rows.length; i++) {
                lines.push(indices[i] + '\t' + stringifyArg(rows[i]));
            }
            __consoleWrite('info', lines.join('\n'));
            return;
        }

        const header = '(index)\t' + keys.join('\t');
        const lines = [header];
        for (let i = 0; i < rows.length; i++) {
            const row = rows[i];
            const vals = keys.map(k => {
                if (row !== null && row !== undefined && typeof row === 'object') {
                    return row[k] !== undefined ? stringifyArg(row[k]) : '';
                }
                return stringifyArg(row);
            });
            lines.push(indices[i] + '\t' + vals.join('\t'));
        }
        __consoleWrite('info', lines.join('\n'));
    },
};

globalThis.console = console;

export { };
