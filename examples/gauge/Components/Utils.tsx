import { ComponentProps, DisplayComponent, FSComponent, Subscribable, VNode } from "@microsoft/msfs-sdk"

type Page = [number, VNode]

interface InterfaceSwitchProps extends ComponentProps {
  class?: string
  intoClass?: string
  active: Subscribable<number>
  pages: Page[]
}

interface InterfaceSwitchPageProps extends ComponentProps {
  class?: string
  active: Subscribable<boolean>
}

export class InterfaceSwitch extends DisplayComponent<InterfaceSwitchProps> {
  private readonly visibility = (pageNumber: number) => this.props.active.map(val => val === pageNumber)

  render(): VNode {
    return (
      <div class={`size-full ${this.props.class ?? "bg-inherit"}`}>
        {this.props.pages.map(([pageNumber, page]) => (
          <InterfaceSwitchPage class={this.props.intoClass ?? "bg-inherit"} active={this.visibility(pageNumber)}>
            {page}
          </InterfaceSwitchPage>
        ))}
      </div>
    )
  }
}

class InterfaceSwitchPage extends DisplayComponent<InterfaceSwitchPageProps> {
  private readonly activeCss = this.props.active.map(val => `size-full p-6 ${val ? "block" : "hidden"}`)

  render(): VNode {
    return <div class={this.activeCss}>{this.props.children}</div>
  }
}

interface InterfaceNavbarProps extends ComponentProps {
  tabs: [number, string][]
  setActive: (pageNumber: number) => void
  active: Subscribable<number>
  class?: string
  intoClass?: string
  activeClass?: string
}

interface InterfaceNavbarItemProps extends ComponentProps {
  content: string
  active: Subscribable<boolean>
  setActive: () => void
  class?: string
  activeClass?: string
}

export class InterfaceNavbar extends DisplayComponent<InterfaceNavbarProps> {
  render(): VNode {
    return (
      <div class={this.props.class}>
        {this.props.tabs.map(([pageNumber, content]) => (
          <InterfaceNavbarItem
            active={this.props.active.map(val => val === pageNumber)}
            setActive={() => this.props.setActive(pageNumber)}
            class="p-4 bg-inherit hover:text-blue-25 text-2xl text-center align-middle"
            activeClass="text-blue-25"
            content={content}
          />
        ))}
      </div>
    )
  }
}

class InterfaceNavbarItem extends DisplayComponent<InterfaceNavbarItemProps> {
  private readonly activeCss = this.props.active.map(
    val => `${this.props.class ?? "size-full"} ${val ? this.props.activeClass ?? "" : ""}`,
  )

  render(): VNode {
    return (
      <Button onClick={this.props.setActive}>
        <p class={this.activeCss}>{this.props.content}</p>
      </Button>
    )
  }
}

interface ButtonProps extends ComponentProps {
  class?: Subscribable<string>
  onClick: () => void
}

export class Button extends DisplayComponent<ButtonProps> {
  private readonly buttonRef = FSComponent.createRef<HTMLDivElement>()

  private readonly class = this.props.class?.map(val => val ?? "text-inherit")

  onAfterRender(node: VNode): void {
    super.onAfterRender(node)

    this.buttonRef.instance.addEventListener("click", this.props.onClick)
  }

  render(): VNode {
    return (
      <div class={this.class} ref={this.buttonRef}>
        {this.props.children}
      </div>
    )
  }
}