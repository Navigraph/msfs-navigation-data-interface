import { ComponentProps, DisplayComponent, FSComponent, Subscribable, UUID, VNode } from "@microsoft/msfs-sdk"

interface InputProps extends ComponentProps {
  value?: string
  class?: string | Subscribable<string>
  textarea?: boolean
}

export class Input extends DisplayComponent<InputProps> {
  private readonly inputId = UUID.GenerateUuid()
  private readonly inputRef = FSComponent.createRef<HTMLInputElement>()

  get value() {
    return this.inputRef.instance.value
  }

  onAfterRender(node: VNode): void {
    super.onAfterRender(node)

    this.inputRef.instance.onfocus = this.onInputFocus
    this.inputRef.instance.onblur = this.onInputBlur
  }

  private getInputProps() {
    return { value: this.props.value, class: this.props.class }
  }

  /**
   * Method to handle when input focus is set
   * @param e The focus event.
   */
  private onInputFocus = (e: FocusEvent): void => {
    e.preventDefault()

    Coherent.trigger("FOCUS_INPUT_FIELD", this.inputId, "", "", this.inputRef.instance.value, false)
    Coherent.on("mousePressOutsideView", () => this.inputRef.instance.blur())
  }

  /**
   * Method to handle on input blur
   */
  private onInputBlur = (): void => {
    Coherent.trigger("UNFOCUS_INPUT_FIELD", "")
    Coherent.off("mousePressOutsideView")
  }

  render() {
    if (this.props.textarea)
      return (
        <textarea style="width:350px;height:100px;" ref={this.inputRef} {...this.getInputProps()}>
          {this.props.value}
        </textarea>
      )
    return <input ref={this.inputRef} {...this.getInputProps()} />
  }
}
