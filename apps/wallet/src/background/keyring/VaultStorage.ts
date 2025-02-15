// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { getRandomEntropy, toEntropy } from '_shared/utils/bip39';
import { fromExportedKeypair } from '_shared/utils/from-exported-keypair';
import { getRandomPassword, makeEphemeraPassword } from '_src/shared/cryptography/keystore';
import type { ExportedKeypair } from '@mysten/sui.js/cryptography';

import {
	getFromLocalStorage,
	getFromSessionStorage,
	isSessionStorageSupported,
	setToLocalStorage,
	setToSessionStorage,
} from '../storage-utils';
import { type DerivedAccount } from './DerivedAccount';
import { type ImportedAccount } from './ImportedAccount';
import { Vault } from './Vault';
import type { StoredData } from './Vault';

const VAULT_KEY = 'vault';
export const EPHEMERAL_PASSWORD_KEY = '244e4b24e667ebf';
export const EPHEMERAL_VAULT_KEY = 'a8e451b8ae8a1b4';

/**
 * @deprecated
 */
class VaultStorageClass {
	#vault: Vault | null = null;

	/**
	 * See {@link Keyring.createVault}
	 * @param password
	 * @param importedEntropy
	 */
	public async create(password: string, importedEntropy?: string) {
		if (await this.isWalletInitialized()) {
			throw new Error(
				'Mnemonic already exists, creating a new one will override it. Clear the existing one first.',
			);
		}
		let vault: Vault | null = new Vault(
			importedEntropy ? toEntropy(importedEntropy) : getRandomEntropy(),
		);
		await setToLocalStorage(VAULT_KEY, await vault.encrypt(password));
		vault = null;
	}

	public async unlock(password: string) {
		const encryptedVault = await getFromLocalStorage<StoredData>(VAULT_KEY);
		if (!encryptedVault) {
			throw new Error('Wallet is not initialized. Create a new one first.');
		}
		this.#vault = await Vault.from(password, encryptedVault, async (aVault) =>
			setToLocalStorage(VAULT_KEY, await aVault.encrypt(password)),
		);
		await this.updateSessionStorage();
	}

	public async lock() {
		this.#vault = null;
		await setToSessionStorage(EPHEMERAL_PASSWORD_KEY, null);
		await setToSessionStorage(EPHEMERAL_VAULT_KEY, null);
	}

	public async revive(): Promise<boolean> {
		let unlocked = false;
		const rndPass = await getFromSessionStorage<string>(EPHEMERAL_PASSWORD_KEY);
		if (rndPass) {
			const ephemeralPass = makeEphemeraPassword(rndPass);
			const ephemeralEncryptedVault = await getFromSessionStorage<StoredData>(EPHEMERAL_VAULT_KEY);
			if (ephemeralEncryptedVault) {
				this.#vault = await Vault.from(ephemeralPass, ephemeralEncryptedVault);
				unlocked = true;
			}
		}
		return unlocked;
	}

	public async clear() {
		await this.lock();
		await setToLocalStorage(VAULT_KEY, null);
	}

	public async isWalletInitialized() {
		return !!(await getFromLocalStorage<StoredData>(VAULT_KEY));
	}

	public get entropy() {
		return this.#vault?.entropy || null;
	}

	public get mnemonicSeedHex() {
		return this.#vault?.mnemonicSeedHex || null;
	}

	public getMnemonicSeedHex() {
		return this.#vault?.mnemonicSeedHex || null;
	}

	public async verifyPassword(password: string) {
		const encryptedVault = await getFromLocalStorage<StoredData>(VAULT_KEY);
		if (!encryptedVault) {
			throw new Error('Wallet is not initialized');
		}
		try {
			await Vault.from(password, encryptedVault);
			return true;
		} catch (e) {
			return false;
		}
	}

	/**
	 * Import a new keypair to the vault and saves it to storage. If keypair already exists it ignores it.
	 * NOTE: make sure you verify the password before calling this method
	 * @param keypair The keypair to import
	 * @param password The password to be used to store the vault. Make sure to verify that it's the correct password (of the current vault) and then call this function. It doesn't verify the password see {@link VaultStorage.verifyPassword}.
	 * @param existingAccounts The wallet's derived and imported accounts
	 * @returns The keyPair if the key was imported, null otherwise
	 */
	public async importKeypair(
		keypair: ExportedKeypair,
		password: string,
		existingAccounts: (ImportedAccount | DerivedAccount)[],
	) {
		if (!this.#vault) {
			throw new Error('Error, vault is locked. Unlock the vault first.');
		}
		const keypairToImport = fromExportedKeypair(keypair);
		const importedAddress = keypairToImport.getPublicKey().toSuiAddress();
		const isDuplicate = existingAccounts.some((anAccount) => anAccount.address === importedAddress);
		if (isDuplicate) {
			return null;
		}
		this.#vault.importedKeypairs.push(keypairToImport);
		await setToLocalStorage(VAULT_KEY, await this.#vault.encrypt(password));
		await this.updateSessionStorage();
		return keypairToImport;
	}

	public getImportedKeys() {
		return this.#vault?.importedKeypairs || null;
	}

	public async storeQredoToken(qredoID: string, token: string, password: string) {
		if (!this.#vault) {
			throw new Error('Error, vault is locked. Unlock the vault first.');
		}
		if (!(await this.verifyPassword(password))) {
			throw new Error('Error, wrong password');
		}
		this.#vault.qredoTokens.set(qredoID, token);
		await setToLocalStorage(VAULT_KEY, await this.#vault.encrypt(password));
		await this.updateSessionStorage();
	}

	public getQredoToken(qredoID: string) {
		if (!this.#vault) {
			throw new Error('Error, vault is locked. Unlock the vault first.');
		}
		return this.#vault.qredoTokens.get(qredoID) || null;
	}

	private async updateSessionStorage() {
		if (!this.#vault || !isSessionStorageSupported()) {
			return;
		}
		const rndPass = getRandomPassword();
		const ephemeralPass = makeEphemeraPassword(rndPass);
		await setToSessionStorage(EPHEMERAL_PASSWORD_KEY, rndPass);
		await setToSessionStorage(EPHEMERAL_VAULT_KEY, await this.#vault.encrypt(ephemeralPass));
	}
}

/**
 * @deprecated
 */
export const VaultStorage = new VaultStorageClass();
