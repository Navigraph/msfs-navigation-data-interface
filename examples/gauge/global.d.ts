interface CoherentEngine {
  /**
   * Asynchronously call a C++ handler and retrieve the result
   * @param name name of the C++ handler to be called
   * @param args any extra parameters to be passed to the C++ handler
   * @return promise for the result of the C++ function
   */
  call(name: "PLAY_INSTRUMENT_SOUND", soundName: string): Promise<void>
  call(name: string, ...args: unknown[]): Promise<unknown>

  on(name: "SetInputTextFromOS" | "mousePressOutsideView", cb: () => void): void
  off(name: "SetInputTextFromOS" | "mousePressOutsideView", cb?: () => void): void

  trigger(name: "FOCUS_INPUT_FIELD" | "UNFOCUS_INPUT_FIELD", ...args: unknown[])
}
