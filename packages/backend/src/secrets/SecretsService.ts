import * as fs from "fs";
import axios from "axios";

const SECRETS_MANAGER_PREFIX = "sm://";

const AWS_SECRETS_EXTENTION_HTTP_PORT = 2773
const AWS_SECRETS_EXTENTION_SERVER_ENDPOINT = `http://localhost:${AWS_SECRETS_EXTENTION_HTTP_PORT}/secretsmanager/get?secretId=`;

export class FileReadError extends Error {
	constructor(path: string) {
		super(`FileReadError: ${path}`);
	}
}

export class FileParseError extends Error {
	constructor(e: unknown) {
		super(`SecretService FileParseError: ${e}`);
	}
}

export class SecretsService {
	private readonly secrets: Map<string, string>;

	constructor(secrets: Map<string, string>) {
		this.secrets = secrets;
	}

	static async fromSecretsManager(secretName: string): Promise<string> {
		const url = `${AWS_SECRETS_EXTENTION_SERVER_ENDPOINT}${secretName}`
    const response = await axios.get(url, {
      headers: {
				'X-Aws-Parameters-Secrets-Token': process.env.AWS_SESSION_TOKEN ?? ''
			}
    });
		const secret = response.data?.SecretString;
		if (!secret) {
			throw new Error(`Secret value for ${secretName} not found!`)
		}
		return secret;
	}

	static async fromEnvVarJsonFile(envName: string, pathToJsonFile: string): Promise<SecretsService> {
		const merged = [];

		let ssFromEnv, ssFromFile: SecretsService;
		try {
			ssFromEnv = SecretsService.fromEnvVar(envName);
			merged.push(...ssFromEnv.getAll().entries());
		} catch (e) {
			console.log("SecretService.fromEnvVarJsonFile: Ignored EnvVar");
		}

		try {
			ssFromFile = SecretsService.fromJsonFile(pathToJsonFile);
			merged.push(...ssFromFile.getAll().entries());
		} catch (e) {
			console.log("SecretService.fromEnvVarJsonFile: Ignored file");
		}

		if (merged.length === 0) {
			throw new Error("No secret found");
		}
		for (const secret of merged) {
			const [, value] = secret
			if (value.startsWith(SECRETS_MANAGER_PREFIX)) {
				const secretName = value.replace(SECRETS_MANAGER_PREFIX, "");
				const secretValue = await this.fromSecretsManager(secretName);
				secret[1] = secretValue;
			}
		}
		return new SecretsService(new Map<string, string>(merged));
	}

	static fromJsonFile(pathToJsonFile: string): SecretsService {
		let fileContents;
		try {
			fileContents = JSON.parse(fs.readFileSync(pathToJsonFile).toString());
		} catch (e) {
			throw new FileReadError(pathToJsonFile);
		}

		const secrets = SecretsService.fromJsonData(fileContents);

		return new SecretsService(secrets);
	}

	static fromEnvVar(envName: string): SecretsService {
		const envValue = process.env[envName];
		if (typeof envValue !== "string") {
			throw new Error(`Given env is not defined: ${envName}`);
		}

		let jsonData: Record<string, unknown>;
		try {
			jsonData = JSON.parse(envValue);
		} catch (e) {
			throw new FileParseError(e);
		}

		return new SecretsService(SecretsService.fromJsonData(jsonData));
	}

	private static fromJsonData(content: Record<string, unknown>): Map<string, string> {
		const secrets = new Map<string, string>();
		Object.entries(content).forEach(([key, value]) => {
			if (typeof value !== "string") {
				return;
			}

			secrets.set(key, value as string);
		});

		return secrets;
	}

	get(secretName: string): string {
		const secret = this.secrets.get(secretName);
		if (secret === undefined) {
			throw new Error("Secret missing: " + secretName);
		}

		return secret;
	}

	getAll(): Map<string, string> {
		return new Map(this.secrets);
	}
}
