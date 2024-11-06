import { ComponentProps, DisplayComponent, FSComponent, VNode } from "@microsoft/msfs-sdk"

interface TestPageProps extends ComponentProps {}

export class TestPage extends DisplayComponent<TestPageProps> {
  render(): VNode {
    return (
      <div class="size-full flex flex-col">
        <p class="mb-8 text-4xl">Test</p>
      </div>
    )
  }
}
