/// <reference types="@microsoft/msfs-types/js/common" />

declare class CommBusListener extends ViewListener.ViewListener {
  callWasm(name: string, jsonBuf: string): void
}

declare function RegisterCommBusListener(callback?: () => void): CommBusListener
