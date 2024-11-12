import {
  ComponentProps,
  DisplayComponent,
  FSComponent,
  MappedSubject,
  ObjectSubject,
  Subject,
  VNode,
} from "@microsoft/msfs-sdk"
import { NavigraphNavigationDataInterface } from "@navigraph/msfs-navigation-data-interface"
import { Checkbox, Input } from "../../Input"
import { Button, InterfaceNavbarItemV2, InterfaceSwitch } from "../../Utils"

interface TestPageProps extends ComponentProps {
  interface: NavigraphNavigationDataInterface
}

interface FunctionDescriptor {
  index: number
  arguments: string[]
  name: string
  functionCallback: (input?: string, inputAlt?: string) => Promise<unknown>
}

interface InputState {
  active: boolean
  type: InputStateType
  hint: string
}

enum InputStateType {
  String,
  Bool,
}

export class TestPage extends DisplayComponent<TestPageProps> {
  private readonly functionList: FunctionDescriptor[] = [
    {
      index: 0,
      arguments: ["url: string", "activate: bool"],
      name: "DownloadNavigationData",
      functionCallback: input => this.props.interface.download_navigation_data(input ?? ""),
    },
    {
      index: 1,
      arguments: [],
      name: "GetActivePackage",
      functionCallback: () => this.props.interface.get_active_package(),
    },
    {
      index: 2,
      arguments: ["sort: bool", "filter: bool"],
      name: "ListAvailablePackages",
      functionCallback: (input, inputAlt) =>
        this.props.interface.list_available_packages(this.strToBool(input), this.strToBool(inputAlt)),
    },
    {
      index: 3,
      arguments: ["uuid: string"],
      name: "SetActivePackage",
      functionCallback: input => this.props.interface.download_navigation_data(input ?? ""),
    },
  ]

  private readonly input1 = Subject.create("")
  private readonly input2 = Subject.create("")
  private readonly output = Subject.create("")
  private readonly selectedFunction = Subject.create(0)
  private readonly selectedFunctionObj = this.selectedFunction.map(index => this.functionList[index])
  private readonly input1State = ObjectSubject.create<InputState>({
    active: false,
    type: InputStateType.String,
    hint: this.functionList[this.selectedFunction.get()].arguments[0] ?? "",
  })
  private readonly input2State = ObjectSubject.create<InputState>({
    active: false,
    type: InputStateType.String,
    hint: this.functionList[this.selectedFunction.get()].arguments[1] ?? "",
  })

  private doubleInputCss = MappedSubject.create(
    ([input1, input2]) =>
      `flex flex-row h-16 bg-ng-background-500 items-center p-2 ${input1.active && input2.active ? "space-x-2" : ""}`,
    this.input1State,
    this.input2State,
  )

  private strToBool(input?: string): boolean {
    return input == "true" ? true : false
  }

  private handleFunction = () => {
    const functionObj = this.selectedFunctionObj.get()
    const input1 = this.input1.get()
    const input2 = this.input2.get()

    functionObj
      .functionCallback(input1, input2)
      .then(obj => this.output.set(JSON.stringify(obj, null, 2)))
      .catch(err => this.output.set(JSON.stringify(err, null, 2)))
  }

  onAfterRender(node: VNode): void {
    super.onAfterRender(node)

    this.selectedFunctionObj.map(functionObj => {
      const functionArgCount = functionObj.arguments.length

      switch (functionArgCount) {
        case 1: {
          this.input1State.set("active", true)
          this.input2State.set("active", false)
          break
        }
        case 2: {
          this.input1State.set("active", true)
          this.input2State.set("active", true)
          break
        }
        default: {
          this.input1State.set("active", false)
          this.input2State.set("active", false)
          break
        }
      }

      this.input1.set("")
      this.input2.set("")

      functionObj.arguments.forEach((value, index) => {
        const argumentType = value.includes("bool") ? InputStateType.Bool : InputStateType.String

        switch (index) {
          case 1: {
            this.input2State.set("type", argumentType)
            this.input2State.set("hint", functionObj.arguments[1])
            if (argumentType === InputStateType.Bool) {
              this.input2.set("false")
            }
            break
          }
          default: {
            this.input1State.set("type", argumentType)
            this.input1State.set("hint", functionObj.arguments[0])
            if (argumentType === InputStateType.Bool) {
              this.input1.set("false")
            }
            break
          }
        }
      })
    })
  }

  render(): VNode {
    return (
      <div class="size-full flex flex-col flex-grow">
        <p class="mb-8 text-4xl">Test</p>
        <div class="size-full w-[875px] flex flex-row">
          <div class="w-1/3 flex flex-col">
            <div class="overflow-scroll flex-grow bg-ng-background-500">
              {this.functionList.map(obj => (
                <InterfaceNavbarItemV2
                  content={""}
                  class="w-full p-2 flex flex-col items-start"
                  activeClass="bg-blue-400"
                  active={this.selectedFunction.map(index => index === obj.index)}
                  setActive={() => this.selectedFunction.set(obj.index)}
                >
                  <p class="text-xl">{obj.name}</p>
                  <p class="text-lg">({obj.arguments.join(", ")})</p>
                </InterfaceNavbarItemV2>
              ))}
            </div>
            <div class={this.doubleInputCss}>
              <InterfaceSwitch
                intoNoTheming
                class="flex content-center"
                intoClass="flex-grow flex"
                active={this.input1State.map(obj => (obj.active ? (obj.type === InputStateType.String ? 0 : 1) : 2))}
                pages={[
                  [
                    0,
                    <Input
                      class="text-xl text-black px-1 size-full"
                      value={this.input1}
                      setValue={value => this.input1.set(value)}
                      default={this.input1State.map(obj => obj.hint)}
                    />,
                  ],
                  [
                    1,
                    <div class="flex-grow flex flex-row space-x-2 items-center justify-center">
                      <p class="text-2xl">{this.input1State.map(obj => obj.hint.split(":")[0] + ":")}</p>
                      <Checkbox value={this.input1} setValue={value => this.input1.set(value)} />
                    </div>,
                  ],
                  [
                    2,
                    <div class="flex-grow flex items-center justify-center">
                      <span class="text-2xl">No Inputs</span>
                    </div>,
                  ],
                ]}
              />
              <InterfaceSwitch
                noTheming
                intoNoTheming
                hideLast
                class={this.input2State.map(obj => (obj.active ? "size-full flex content-center" : ""))}
                intoClass="flex-grow flex content-center"
                active={this.input2State.map(obj => (obj.active ? (obj.type === InputStateType.String ? 0 : 1) : 2))}
                pages={[
                  [
                    0,
                    <Input
                      class="text-xl text-black px-1"
                      value={this.input2}
                      setValue={value => this.input2.set(value)}
                      default={this.input2State.map(obj => obj.hint)}
                    />,
                  ],
                  [
                    1,
                    <div class="flex-grow flex flex-row space-x-2 items-center justify-center">
                      <p class="text-2xl">{this.input2State.map(obj => obj.hint.split(":")[0] + ":")}</p>
                      <Checkbox value={this.input2} setValue={value => this.input2.set(value)} />
                    </div>,
                  ],
                  [2, <></>],
                ]}
              />
            </div>
          </div>
          <div class="w-2/3 bg-ng-background-700 flex flex-col flex-grow">
            <div class="size-full p-2 flex-grow overflow-auto">
              <p class="text-xl whitespace-pre">{this.output}</p>
            </div>
            <Button
              class="h-16 flex flex-row items-center pl-4 bg-blue-400 overflow-auto"
              onClick={() => this.handleFunction()}
            >
              <span class="text-3xl">Try:</span>
              <span class="text-2xl pl-4 whitespace-pre">
                {MappedSubject.create(
                  ([obj, input1, input2, input1State, input2State]) =>
                    `${obj.name}(${
                      input1State.active ? (input1State.type === InputStateType.String ? `"${input1}"` : input1) : ""
                    }${
                      input2State.active
                        ? `, ${input2State.type === InputStateType.String ? `"${input2}"` : input2}`
                        : ""
                    })`,
                  this.selectedFunctionObj,
                  this.input1,
                  this.input2,
                  this.input1State,
                  this.input2State,
                )}
              </span>
            </Button>
          </div>
        </div>
      </div>
    )
  }
}
