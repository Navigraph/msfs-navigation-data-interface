import { ComponentProps, DisplayComponent, FSComponent, VNode } from "@microsoft/msfs-sdk"

interface AuthPageProps extends ComponentProps {}

export class AuthPage extends DisplayComponent<AuthPageProps> {
  render(): VNode {
    return (
      <div class="size-full flex flex-col">
        <p class="mb-8 text-4xl">Authentication</p>
      </div>
    )
  }
}
