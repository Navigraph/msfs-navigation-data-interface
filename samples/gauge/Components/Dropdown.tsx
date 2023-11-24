import { ComponentProps, ComputedSubject, DisplayComponent, FSComponent, Subject, VNode } from "@microsoft/msfs-sdk"
import "./Dropdown.css"

export class Dropdown extends DisplayComponent<ComponentProps> {
  private readonly dropdownButtonRef = FSComponent.createRef<HTMLDivElement>()
  private readonly dropdownMenuRef = FSComponent.createRef<HTMLUListElement>()

  private readonly dropdownButtonText = Subject.create<string>("Select an item")

  private navdataFormat: null | string = null

  public render(): VNode {
    return (
      <div class="dropdown">
        <div ref={this.dropdownButtonRef} class="dropdown-toggle">
          {this.dropdownButtonText}
        </div>
        <ul ref={this.dropdownMenuRef} class="dropdown-menu" />
      </div>
    )
  }

  public onAfterRender(node: VNode): void {
    const dropdownButton = this.dropdownButtonRef.instance
    const dropdownMenu = this.dropdownMenuRef.instance

    dropdownButton.addEventListener("click", function () {
      dropdownMenu.style.display = dropdownMenu.style.display === "block" ? "none" : "block"
    })

    // Close the dropdown when clicking outside of it
    document.addEventListener("click", this.onDropdownItemClick.bind(this))
  }

  public onDropdownItemClick(event: Event): void {
    const dropdownButton = this.dropdownButtonRef.instance
    const dropdownMenu = this.dropdownMenuRef.instance

    const target = event.target as HTMLElement
    if (!target) {
      return
    }
    if (!dropdownMenu.contains(target) && !dropdownButton.contains(target)) {
      dropdownMenu.style.display = "none"
    } else if (dropdownMenu.contains(target)) {
      this.dropdownButtonText.set(target.textContent as string)
      const navdataFormat = target.dataset.navdataFormat
      if (navdataFormat) {
        this.navdataFormat = navdataFormat
      }
    }
  }

  public addDropdownItem(text: string, format: string): void {
    const dropdownItem = document.createElement("li")
    dropdownItem.textContent = text
    dropdownItem.dataset.navdataFormat = format
    this.dropdownMenuRef.instance.appendChild(dropdownItem)
  }

  public getNavdataFormat(): string | null {
    return this.navdataFormat
  }
}