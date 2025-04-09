import 'package:flutter/material.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'models/comment.dart';
import 'models/video.dart';
import 'models/user.dart';
import 'services/api_service.dart';
import 'services/auth_service.dart';
import 'screens/home_screen.dart';
import 'screens/login_screen.dart';
import 'screens/splash_screen.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await dotenv.load(fileName: ".env");
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (context) => AuthProvider()),
        ChangeNotifierProxyProvider<AuthProvider, CommentProvider>(
          create: (context) => CommentProvider(null),
          update: (context, auth, previous) => CommentProvider(auth.sessionId),
        ),
        ChangeNotifierProxyProvider<AuthProvider, VideoProvider>(
          create: (context) => VideoProvider(null),
          update: (context, auth, previous) => VideoProvider(auth.sessionId),
        ),
      ],
      child: MaterialApp(
        title: 'YouTube Commenter',
        theme: ThemeData(
          primarySwatch: Colors.red,
          visualDensity: VisualDensity.adaptivePlatformDensity,
          useMaterial3: true,
        ),
        home: const SplashScreen(),
        routes: {
          '/login': (context) => const LoginScreen(),
          '/home': (context) => const HomeScreen(),
        },
      ),
    );
  }
}

class AuthProvider extends ChangeNotifier {
  final AuthService _authService = AuthService();
  String? _sessionId;
  User? _user;
  bool _isLoading = false;
  String? _error;

  AuthProvider() {
    _loadSessionFromPrefs();
  }

  String? get sessionId => _sessionId;
  User? get user => _user;
  bool get isLoading => _isLoading;
  String? get error => _error;
  bool get isAuthenticated => _sessionId != null;

  Future<void> _loadSessionFromPrefs() async {
    _isLoading = true;
    notifyListeners();

    try {
      final prefs = await SharedPreferences.getInstance();
      _sessionId = prefs.getString('sessionId');
      final userJson = prefs.getString('user');
      if (userJson != null) {
        _user = User.fromJson(userJson);
      }
    } catch (e) {
      _error = e.toString();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<String> getAuthUrl() async {
    return await _authService.getAuthUrl();
  }

  Future<void> setSession(String sessionId, User user) async {
    _sessionId = sessionId;
    _user = user;

    // Save to preferences
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('sessionId', sessionId);
    await prefs.setString('user', user.toJson());

    notifyListeners();
  }

  Future<void> logout() async {
    _sessionId = null;
    _user = null;

    // Clear preferences
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove('sessionId');
    await prefs.remove('user');

    notifyListeners();
  }
}

class CommentProvider extends ChangeNotifier {
  final ApiService _apiService = ApiService();
  final String? _sessionId;
  List<Comment> _comments = [];
  String? _currentVideoId;
  bool _isLoading = false;
  String? _error;

  CommentProvider(this._sessionId);

  List<Comment> get comments => _comments;
  String? get currentVideoId => _currentVideoId;
  bool get isLoading => _isLoading;
  String? get error => _error;

  Future<void> fetchComments(String videoId) async {
    if (_sessionId == null) return;

    _isLoading = true;
    _error = null;
    _currentVideoId = videoId;
    notifyListeners();

    try {
      _comments = await _apiService.getComments(videoId, _sessionId!);
      _isLoading = false;
      notifyListeners();
    } catch (e) {
      _isLoading = false;
      _error = e.toString();
      notifyListeners();
    }
  }

  Future<String> generateReply(String commentId, {String tone = 'friendly'}) async {
    if (_sessionId == null) throw Exception('Not authenticated');

    try {
      final response = await _apiService.generateReply(commentId, tone, _sessionId!);
      return response.replyText;
    } catch (e) {
      _error = e.toString();
      notifyListeners();
      rethrow;
    }
  }

  Future<void> postReply(String commentId, String replyText, {bool aiGenerated = false, String? aiModel}) async {
    if (_sessionId == null) throw Exception('Not authenticated');

    try {
      await _apiService.postReply(commentId, replyText, aiGenerated, aiModel, _sessionId!);

      // Refresh comments to show the new reply
      if (_currentVideoId != null) {
        await fetchComments(_currentVideoId!);
      }
    } catch (e) {
      _error = e.toString();
      notifyListeners();
      rethrow;
    }
  }
}

class VideoProvider extends ChangeNotifier {
  final ApiService _apiService = ApiService();
  final String? _sessionId;
  List<Video> _videos = [];
  bool _isLoading = false;
  String? _error;

  VideoProvider(this._sessionId);

  List<Video> get videos => _videos;
  bool get isLoading => _isLoading;
  String? get error => _error;

  Future<void> fetchVideos() async {
    if (_sessionId == null) return;

    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _videos = await _apiService.getVideos(_sessionId!);
      _isLoading = false;
      notifyListeners();
    } catch (e) {
      _isLoading = false;
      _error = e.toString();
      notifyListeners();
    }
  }
}
