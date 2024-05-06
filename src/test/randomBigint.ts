// https://github.com/bnoordhuis/random-bigint/blob/master/index.js

import { randomBytes } from "crypto"

export function random(bits: number) {
	if (bits < 0)
		throw new RangeError('bits < 0')

	// @ts-ignore
	const n = (bits >>> 3) + !!(bits & 7) // Round up to next byte.
	const r = 8*n - bits
	const s = 8 - r
	const m = (1 << s) - 1 // Bits to mask off from MSB.

	const bytes = randomBytes(n)

	maskbits(m, bytes)

	return bytes2bigint(bytes)
}

function maskbits(m: number, bytes: Buffer) {
	// Mask off bits from the MSB that are > log2(bits).
	// |bytes| is treated as a big-endian bigint so byte 0 is the MSB.
	if (bytes.length > 0)
		bytes[0] &= m
}

function bytes2bigint(bytes: Buffer) {
	let result = BigInt(0)

	const n = bytes.length

	// Read input in 8 byte slices. This is, on average and at the time
	// of writing, about 35x faster for large inputs than processing them
	// one byte at a time.
	if (n >= 8) {
		const view = new DataView(bytes.buffer, bytes.byteOffset)

		for (let i = 0, k = n & ~7; i < k; i += 8) {
			const x = view.getBigUint64(i, false)
			result = (result << BigInt(64)) + x
		}
	}

	// Now mop up any remaining bytes.
	for (let i = n & ~7; i < n; i++)
		result = result * BigInt(256) + BigInt(bytes[i])

	return result
}