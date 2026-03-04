import type { EventTypes } from '$lib';

type EventCallback<T = any> = (data: T) => void;

class EventBus {
	private events: Map<EventTypes, EventCallback[]> = new Map();

	/**
	 * Subscribe to an event
	 * @param event Event name
	 * @param callback Callback function to execute when event is emitted
	 * @returns Unsubscribe function
	 */
	on<T = any>(event: EventTypes, callback: EventCallback<T>): () => void {
		if (!this.events.has(event)) {
			this.events.set(event, []);
		}
		this.events.get(event)!.push(callback);

		// Return unsubscribe function
		return () => this.off(event, callback);
	}

	/**
	 * Subscribe to an event that only fires once
	 * @param event Event name
	 * @param callback Callback function to execute when event is emitted
	 */
	once<T = any>(event: EventTypes, callback: EventCallback<T>): void {
		const onceCallback: EventCallback<T> = (data) => {
			callback(data);
			this.off(event, onceCallback);
		};
		this.on(event, onceCallback);
	}

	/**
	 * Unsubscribe from an event
	 * @param event Event name
	 * @param callback Callback function to remove
	 */
	off(event: EventTypes, callback: EventCallback): void {
		const callbacks = this.events.get(event);
		if (callbacks) {
			const index = callbacks.indexOf(callback);
			if (index > -1) {
				callbacks.splice(index, 1);
			}
			// Clean up if no more listeners
			if (callbacks.length === 0) {
				this.events.delete(event);
			}
		}
	}

	/**
	 * Emit an event with optional data
	 * @param event Event name
	 * @param data Optional data to pass to listeners
	 */
	emit<T = any>(event: EventTypes, data?: T): void {
		const callbacks = this.events.get(event);
		if (callbacks) {
			callbacks.forEach((callback) => callback(data));
		}
	}

	/**
	 * Remove all listeners for an event, or all events if no event name provided
	 * @param event Optional event name
	 */
	clear(event?: EventTypes): void {
		if (event) {
			this.events.delete(event);
		} else {
			this.events.clear();
		}
	}

	/**
	 * Get list of all registered events
	 */
	getEvents(): EventTypes[] {
		return Array.from(this.events.keys());
	}
}

// Export singleton instance
export const eventBus = new EventBus();

// Export type for custom event definitions
export type { EventCallback };
