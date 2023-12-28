import { NavigraphNavdataInterface } from "../js"

const navdataInterface = new NavigraphNavdataInterface()

describe("test", () => {
  it("Fetch airport", async () => {
    const airport = await navdataInterface.getAirport("KJFK")

    expect(airport.airport_identifier).toBe("KJFK")
  })
})
