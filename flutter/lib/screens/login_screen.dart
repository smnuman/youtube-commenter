import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../main.dart';

class LoginScreen extends StatelessWidget {
  const LoginScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final authProvider = Provider.of<AuthProvider>(context);

    return Scaffold(
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Image.network(
                'https://www.gstatic.com/youtube/img/branding/youtubelogo/svg/youtubelogo.svg',
                width: 200,
              ),
              const SizedBox(height: 24),
              const Text(
                'YouTube Commenter',
                style: TextStyle(
                  fontSize: 24,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 16),
              const Text(
                'Manage your YouTube comments with AI assistance',
                textAlign: TextAlign.center,
                style: TextStyle(
                  fontSize: 16,
                  color: Colors.grey,
                ),
              ),
              const SizedBox(height: 48),
              ElevatedButton.icon(
                onPressed: authProvider.isLoading
                    ? null
                    : () async {
                        try {
                          await authProvider.getAuthUrl();
                          // In a real app, we would handle the OAuth flow here
                          // For now, we'll just navigate to the home screen
                          if (context.mounted) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(
                                content: Text('OAuth flow would start here'),
                              ),
                            );
                          }
                        } catch (e) {
                          if (context.mounted) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(
                                content: Text('Error: $e'),
                                backgroundColor: Colors.red,
                              ),
                            );
                          }
                        }
                      },
                icon: const Icon(Icons.login),
                label: const Text('Sign in with YouTube'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: Colors.red,
                  foregroundColor: Colors.white,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 24,
                    vertical: 12,
                  ),
                ),
              ),
              if (authProvider.isLoading)
                const Padding(
                  padding: EdgeInsets.only(top: 24),
                  child: CircularProgressIndicator(),
                ),
              if (authProvider.error != null)
                Padding(
                  padding: const EdgeInsets.only(top: 24),
                  child: Text(
                    'Error: ${authProvider.error}',
                    style: const TextStyle(color: Colors.red),
                  ),
                ),
              // For development purposes, add a button to skip login
              const SizedBox(height: 24),
              TextButton(
                onPressed: () {
                  Navigator.of(context).pushReplacementNamed('/home');
                },
                child: const Text('Skip login (for development)'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
