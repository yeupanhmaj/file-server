import axios, { type AxiosInstance, type AxiosRequestConfig } from 'axios';

export class ServiceBase {
	protected axiosInstance: AxiosInstance;

	constructor() {
		const baseURL = import.meta.env.VITE_API_URL;
		this.axiosInstance = axios.create({
			baseURL
			// Don't set default Content-Type - let each request set its own
		});
	}

	protected async get<T = unknown>(url: string, config?: AxiosRequestConfig) {
		const response = await this.axiosInstance.get<T>(url, config);
		return response.data;
	}

	protected async post<T = unknown>(url: string, data?: unknown, config?: AxiosRequestConfig) {
		const response = await this.axiosInstance.post<T>(url, data, config);
		return response.data;
	}

	protected async put<T = unknown>(url: string, data?: unknown, config?: AxiosRequestConfig) {
		const response = await this.axiosInstance.put<T>(url, data, config);
		return response.data;
	}

	protected async delete<T = unknown>(url: string, config?: AxiosRequestConfig) {
		const response = await this.axiosInstance.delete<T>(url, config);
		return response.data;
	}
}
