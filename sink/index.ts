import * as url from 'node:url'
import { createGrpcTransport } from '@connectrpc/connect-node'
import { createAuthInterceptor, createRegistry, createRequest, streamBlocks, unpackMapOutput } from '@substreams/core'
import { readPackage } from '@substreams/manifest'

const SUBSTREAM = process.env.SPKG || url.fileURLToPath(new URL('../tokens/solana-tokens-v0.1.0.spkg', import.meta.url))
const MODULE = process.env.MODULE || 'db_out'
const TOKEN = mustEnv('SUBSTREAMS_API_TOKEN')

async function main() {
  const substream = await readPackage(SUBSTREAM)
  const registry = createRegistry(substream)
  const transport = createGrpcTransport({
    baseUrl: 'https://mainnet.sol.streamingfast.io',
    httpVersion: '2',
    interceptors: [createAuthInterceptor(TOKEN)],
    jsonOptions: {
      typeRegistry: registry,
    },
  })

  const request = createRequest({
    substreamPackage: substream,
    outputModule: MODULE,
    productionMode: true,
    startBlockNum: 200_000_000,
    stopBlockNum: '+1000000',
  })

  console.log('Streaming blocks...')
  let start = Date.now()

  for await (const response of streamBlocks(transport, request)) {
    const output = unpackMapOutput(response, registry)

    if (response.message.case === 'blockScopedData' && Number(response.message.value.clock?.number) % 10000 === 0) {
      console.log(`Received block #${response.message.value.clock?.number || 0} (${output?.getType().name})`)
    }
  }

  console.log(`Time elapsed: ${(Date.now() - start) / 10_000}ms/block (Total ${Date.now() - start}ms)`)
}

function mustEnv(name: string): string {
  const value = process.env[name]
  if (value === undefined) {
    throw new Error(`${name} is required`)
  }

  return value
}

main().catch(console.error)
