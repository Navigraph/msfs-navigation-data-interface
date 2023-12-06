declare class CommBusListener extends ViewListener.ViewListener {
    callWasm(name: string, jsonBuf: string): void;
}
declare function RegisterCommBusListener(callback?: any): CommBusListener;