import "@microsoft/msfs-sdk";

declare module "@microsoft/msfs-sdk" {
  namespace FSComponent {
    namespace JSX {
      type Element = import("@microsoft/msfs-sdk").VNode;
    }
  }
}
