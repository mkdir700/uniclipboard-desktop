use async_trait::async_trait;
use rand::RngCore;

use argon2::Argon2;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use uc_core::ports::EncryptionPort;
use uc_core::security::model::{
    AeadAlgorithm, EncryptedBlob, EncryptionError, EncryptionFormatVersion, KdfAlgorithm,
    KdfParams, Kek, MasterKey, Passphrase,
};

pub struct EncryptionRepository;

const CURR_VERSION: EncryptionFormatVersion = EncryptionFormatVersion::V1;

fn aad_fingerprint(aad: &[u8]) -> Vec<u8> {
    blake3::hash(aad).as_bytes()[..16].to_vec()
}

#[async_trait]
impl EncryptionPort for EncryptionRepository {
    async fn derive_kek(
        &self,
        passphrase: &Passphrase,
        salt: &[u8],
        kdf: &KdfParams,
    ) -> Result<Kek, EncryptionError> {
        match kdf.alg {
            KdfAlgorithm::Argon2id => {
                let argon2 = Argon2::new(
                    argon2::Algorithm::Argon2id,
                    argon2::Version::V0x13,
                    argon2::Params::new(
                        kdf.params.mem_kib,
                        kdf.params.iters,
                        kdf.params.parallelism,
                        Some(32),
                    )
                    .map_err(|_| EncryptionError::InvalidParameter(format!("{:?}", kdf.params)))?,
                );

                let mut okm = [0u8; 32];
                argon2
                    .hash_password_into(passphrase.as_bytes(), salt, &mut okm)
                    .map_err(|_| EncryptionError::KdfFailed)?;

                Kek::from_bytes(&okm)
            }
        }
    }
    async fn wrap_master_key(
        &self,
        kek: &Kek,
        master_key: &MasterKey,
        aead: AeadAlgorithm,
    ) -> Result<EncryptedBlob, EncryptionError> {
        let mut nonce = vec![0u8; 24];
        rand::rng().fill_bytes(&mut nonce);

        let ciphertext = match aead {
            AeadAlgorithm::XChaCha20Poly1305 => {
                let cipher = XChaCha20Poly1305::new_from_slice(kek.as_bytes())
                    .map_err(|_| EncryptionError::InvalidKey)?;
                cipher
                    .encrypt(XNonce::from_slice(&nonce), master_key.as_bytes())
                    .map_err(|_| EncryptionError::EncryptFailed)?
            }
        };

        Ok(EncryptedBlob {
            version: CURR_VERSION,
            nonce,
            ciphertext,
            aead,
            aad_fingerprint: None,
        })
    }

    async fn unwrap_master_key(
        &self,
        kek: &Kek,
        wrapped: &EncryptedBlob,
    ) -> Result<MasterKey, EncryptionError> {
        let plaintext = match wrapped.aead {
            AeadAlgorithm::XChaCha20Poly1305 => {
                let cipher = XChaCha20Poly1305::new_from_slice(kek.as_bytes())
                    .map_err(|_| EncryptionError::InvalidKey)?;
                cipher
                    .decrypt(
                        XNonce::from_slice(&wrapped.nonce),
                        wrapped.ciphertext.as_ref(),
                    )
                    .map_err(|_| EncryptionError::WrongPassphrase)?
            }
        };

        MasterKey::from_bytes(&plaintext)
    }

    async fn encrypt_blob(
        &self,
        master_key: &MasterKey,
        plaintext: &[u8],
        aad: &[u8],
        aead: AeadAlgorithm,
    ) -> Result<EncryptedBlob, EncryptionError> {
        let mut nonce = vec![0u8; 24];
        rand::thread_rng().fill_bytes(&mut nonce);

        let ciphertext = match aead {
            AeadAlgorithm::XChaCha20Poly1305 => {
                let cipher = XChaCha20Poly1305::new_from_slice(master_key.as_bytes())
                    .map_err(|_| EncryptionError::InvalidKey)?;
                cipher
                    .encrypt(
                        XNonce::from_slice(&nonce),
                        chacha20poly1305::aead::Payload {
                            msg: plaintext,
                            aad,
                        },
                    )
                    .map_err(|_| EncryptionError::EncryptFailed)?
            }
        };

        let aad_fp = Some(aad_fingerprint(aad));

        Ok(EncryptedBlob {
            version: EncryptionFormatVersion::V1,
            nonce,
            ciphertext,
            aead,
            aad_fingerprint: aad_fp,
        })
    }

    async fn decrypt_blob(
        &self,
        master_key: &MasterKey,
        encrypted: &EncryptedBlob,
        aad: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        let plaintext = match encrypted.aead {
            AeadAlgorithm::XChaCha20Poly1305 => {
                let cipher = XChaCha20Poly1305::new_from_slice(master_key.as_bytes())
                    .map_err(|_| EncryptionError::InvalidKey)?;
                cipher
                    .decrypt(
                        XNonce::from_slice(&encrypted.nonce),
                        chacha20poly1305::aead::Payload {
                            msg: encrypted.ciphertext.as_ref(),
                            aad,
                        },
                    )
                    .map_err(|_| EncryptionError::CorruptedBlob)?
            }
        };

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::ports::EncryptionPort;
    use uc_core::security::model::{
        AeadAlgorithm, EncryptionError, KdfAlgorithm, KdfParams, KdfParamsV1, Kek, MasterKey,
        Passphrase,
    };

    fn test_kdf_params() -> KdfParams {
        KdfParams {
            alg: KdfAlgorithm::Argon2id,
            params: KdfParamsV1 {
                mem_kib: 32,
                iters: 1,
                parallelism: 1,
            },
        }
    }

    #[tokio::test]
    async fn derive_kek_is_deterministic() {
        let service = EncryptionRepository;
        let passphrase = Passphrase("test-passphrase".to_string());
        let salt = b"salt-000000000000";
        let kdf = test_kdf_params();

        let k1 = service
            .derive_kek(&passphrase, salt, &kdf)
            .await
            .expect("derive kek");
        let k2 = service
            .derive_kek(&passphrase, salt, &kdf)
            .await
            .expect("derive kek");

        assert_eq!(k1, k2);
    }

    #[tokio::test]
    async fn derive_kek_changes_with_salt() {
        let service = EncryptionRepository;
        let passphrase = Passphrase("test-passphrase".to_string());
        let kdf = test_kdf_params();
        let salt_a = b"salt-aaaaaaaaaaaa";
        let salt_b = b"salt-bbbbbbbbbbbb";

        let k1 = service
            .derive_kek(&passphrase, salt_a, &kdf)
            .await
            .expect("derive kek");
        let k2 = service
            .derive_kek(&passphrase, salt_b, &kdf)
            .await
            .expect("derive kek");

        assert_ne!(k1, k2);
    }

    #[tokio::test]
    async fn wrap_and_unwrap_master_key_round_trip() {
        let service = EncryptionRepository;
        let kek = Kek([1u8; 32]);
        let master_key = MasterKey([2u8; 32]);

        let wrapped = service
            .wrap_master_key(&kek, &master_key, AeadAlgorithm::XChaCha20Poly1305)
            .await
            .expect("wrap master key");

        assert_eq!(wrapped.version, CURR_VERSION);
        assert_eq!(wrapped.nonce.len(), 24);
        assert!(!wrapped.ciphertext.is_empty());

        let unwrapped = service
            .unwrap_master_key(&kek, &wrapped)
            .await
            .expect("unwrap master key");

        assert_eq!(unwrapped, master_key);
    }

    #[tokio::test]
    async fn unwrap_master_key_wrong_kek_returns_wrong_passphrase() {
        let service = EncryptionRepository;
        let kek = Kek([1u8; 32]);
        let wrong_kek = Kek([9u8; 32]);
        let master_key = MasterKey([2u8; 32]);

        let wrapped = service
            .wrap_master_key(&kek, &master_key, AeadAlgorithm::XChaCha20Poly1305)
            .await
            .expect("wrap master key");

        let err = service
            .unwrap_master_key(&wrong_kek, &wrapped)
            .await
            .expect_err("expected WrongPassphrase");

        assert!(matches!(err, EncryptionError::WrongPassphrase));
    }

    #[tokio::test]
    async fn encrypt_then_decrypt_round_trip_with_aad() {
        let service = EncryptionRepository;
        let master_key = MasterKey([3u8; 32]);
        let plaintext = b"hello-uniclipboard";
        let aad = b"aad-context";

        let encrypted = service
            .encrypt_blob(
                &master_key,
                plaintext,
                aad,
                AeadAlgorithm::XChaCha20Poly1305,
            )
            .await
            .expect("encrypt blob");

        assert_eq!(encrypted.nonce.len(), 24);
        assert_eq!(encrypted.aad_fingerprint, Some(aad_fingerprint(aad)));

        let decrypted = service
            .decrypt_blob(&master_key, &encrypted, aad)
            .await
            .expect("decrypt blob");

        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn decrypt_blob_wrong_aad_returns_corrupted_blob() {
        let service = EncryptionRepository;
        let master_key = MasterKey([3u8; 32]);
        let plaintext = b"hello-uniclipboard";
        let aad = b"aad-context";

        let encrypted = service
            .encrypt_blob(
                &master_key,
                plaintext,
                aad,
                AeadAlgorithm::XChaCha20Poly1305,
            )
            .await
            .expect("encrypt blob");

        let err = service
            .decrypt_blob(&master_key, &encrypted, b"wrong-aad")
            .await
            .expect_err("expected CorruptedBlob");

        assert!(matches!(err, EncryptionError::CorruptedBlob));
    }

    #[tokio::test]
    async fn decrypt_blob_wrong_key_returns_corrupted_blob() {
        let service = EncryptionRepository;
        let master_key = MasterKey([3u8; 32]);
        let wrong_key = MasterKey([4u8; 32]);
        let plaintext = b"hello-uniclipboard";
        let aad = b"aad-context";

        let encrypted = service
            .encrypt_blob(
                &master_key,
                plaintext,
                aad,
                AeadAlgorithm::XChaCha20Poly1305,
            )
            .await
            .expect("encrypt blob");

        let err = service
            .decrypt_blob(&wrong_key, &encrypted, aad)
            .await
            .expect_err("expected CorruptedBlob");

        assert!(matches!(err, EncryptionError::CorruptedBlob));
    }
}
