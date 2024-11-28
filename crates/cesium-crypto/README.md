# cesium-crypto

### Dilithium (ML-DSA)

Currently the only standardised and implemented scheme for post-quantum public key cryptography. Current implementation is using dilithium2 (mldsa-44; NIST level 2) instead of dilithium3 (mldsa-65; NIST level 3). In the future this will change to either mldsa-65 or fn-dsa if the standardisation and implementation comes through.

### Falcon

Awaiting standardisation (FN-DSA, FFT) by NIST and subsequent implementation of FIPS-206 in the pycrypto package. Current implementation is using Falcon-1024 (NIST level 5).

### Sphincs+

Awaiting standardised implementation (SLH-DSA) of FIPS-205 in the pqcrypto package.
