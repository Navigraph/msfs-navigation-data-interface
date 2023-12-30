import { NavigraphNavdataInterface } from "../js"

const navdataInterface = new NavigraphNavdataInterface()

describe("test", () => {
  it("Fetch airport", async () => {
    const airport = await navdataInterface.getAirport("KJFK")

    expect(airport.ident).toBe("KJFK")
  })

  it("Get airports in range", async () => {
    const airports = await navdataInterface.getAirportsInRange({ lat: 51.468, long: -0.4551 }, 640)

    expect(airports.length).toBe(1686)
  })
})
