import { DisplayComponent, FSComponent, VNode } from "@microsoft/msfs-sdk"

export class MyComponent extends DisplayComponent<any> {
  public render(): VNode {
    return <div class="my-component">Hello World!</div>
  }
}
