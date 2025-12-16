package com.example;

import net.fabricmc.api.ClientModInitializer;

public class ExampleModClient implements ClientModInitializer {
	@Override
	public void onInitializeClient() {
		System.out.println("Client initialized!");
		// This entrypoint is suitable for setting up client-specific logic, such as rendering.
	}
}