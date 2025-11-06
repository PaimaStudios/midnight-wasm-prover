import { createShieldedCoinInfo, sampleCoinPublicKey, sampleEncryptionPublicKey, ShieldedTokenType, Transaction, ZswapOffer, ZswapOutput } from "@midnight-ntwrk/ledger-v6";

export const generateHex = (len: number) =>
  [...Array(len)].map(() => Math.floor(Math.random() * 16).toString(16)).join('');

const sampleShieldedTokenType = (): ShieldedTokenType => ({
  tag: 'shielded',
  raw: generateHex(64)
});

const sampleBigInt = () => {
  return (
    BigInt(Math.floor(Math.random() * Number.MAX_SAFE_INTEGER)) *
    BigInt(Math.floor(Math.random() * Number.MAX_SAFE_INTEGER))
  );
};

const unprovenOfferFromOutput = (
  segment: number = 0,
  tokenType: ShieldedTokenType = sampleShieldedTokenType(),
  value: bigint = sampleBigInt(),
  targetCpk: string = sampleCoinPublicKey(),
  targetEpk: string = sampleEncryptionPublicKey()
) => {
  return ZswapOffer.fromOutput(
    ZswapOutput.new(createShieldedCoinInfo(tokenType.raw, value), segment, targetCpk, targetEpk),
    tokenType.raw,
    value
  );
};

export const unprovenTransactionGuaranteed = () => {
  return Transaction.fromParts('local-test', unprovenOfferFromOutput());
};


export const unprovenTransactionGuaranteedAndFallible = () => {
  return Transaction.fromParts('local-test', unprovenOfferFromOutput(), unprovenOfferFromOutput(1));
};

export default {
  unprovenTransactionGuaranteedAndFallible,
  unprovenTransactionGuaranteed
};

