using System;
using System.Runtime.InteropServices;
using System.Text;

namespace SpeechMarkdown
{
    public class SpeechMarkdownParser : IDisposable
    {
        private static readonly object _lock = new object();

        private IntPtr _libraryHandle;

        private const string DllName = "speechmarkdown_rust";

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr speechmarkdown_to_ssml(IntPtr input, IntPtr platform);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr speechmarkdown_to_text(IntPtr input);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr speechmarkdown_parse(IntPtr input);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern void speechmarkdown_free(IntPtr s);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr speechmarkdown_get_error();

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        [return: MarshalAs(UnmanagedType.I1)]
        private static extern bool speechmarkdown_is_speech_markdown(IntPtr input);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        [return: MarshalAs(UnmanagedType.I1)]
        private static extern bool speechmarkdown_validate(IntPtr input);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr speechmarkdown_to_smd(IntPtr input);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr speechmarkdown_supported_ssml(IntPtr platform);

        public SpeechMarkdownParser()
        {
        }

        public string ToSsml(string input, string platform)
        {
            if (input == null) throw new ArgumentNullException(nameof(input));
            if (platform == null) throw new ArgumentNullException(nameof(platform));

            lock (_lock)
            {
                IntPtr inputPtr = MarshalUtf8(input);
                IntPtr platformPtr = MarshalUtf8(platform);

                try
                {
                    IntPtr resultPtr = speechmarkdown_to_ssml(inputPtr, platformPtr);

                    if (resultPtr == IntPtr.Zero)
                    {
                        string error = GetAndFreeError();
                        throw new SpeechMarkdownException(error ?? "Unknown error");
                    }

                    return PtrToStringAndFree(resultPtr);
                }
                finally
                {
                    Marshal.FreeHGlobal(inputPtr);
                    Marshal.FreeHGlobal(platformPtr);
                }
            }
        }

        public string ToText(string input)
        {
            if (input == null) throw new ArgumentNullException(nameof(input));

            lock (_lock)
            {
                IntPtr inputPtr = MarshalUtf8(input);

                try
                {
                    IntPtr resultPtr = speechmarkdown_to_text(inputPtr);

                    if (resultPtr == IntPtr.Zero)
                    {
                        string error = GetAndFreeError();
                        throw new SpeechMarkdownException(error ?? "Unknown error");
                    }

                    return PtrToStringAndFree(resultPtr);
                }
                finally
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        public string ParseToJson(string input)
        {
            if (input == null) throw new ArgumentNullException(nameof(input));

            lock (_lock)
            {
                IntPtr inputPtr = MarshalUtf8(input);

                try
                {
                    IntPtr resultPtr = speechmarkdown_parse(inputPtr);

                    if (resultPtr == IntPtr.Zero)
                    {
                        string error = GetAndFreeError();
                        throw new SpeechMarkdownException(error ?? "Unknown error");
                    }

                    return PtrToStringAndFree(resultPtr);
                }
                finally
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        public bool IsSpeechMarkdown(string input)
        {
            if (input == null) throw new ArgumentNullException(nameof(input));

            lock (_lock)
            {
                IntPtr inputPtr = MarshalUtf8(input);

                try
                {
                    return speechmarkdown_is_speech_markdown(inputPtr);
                }
                finally
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        public bool Validate(string input)
        {
            if (input == null) throw new ArgumentNullException(nameof(input));

            lock (_lock)
            {
                IntPtr inputPtr = MarshalUtf8(input);

                try
                {
                    bool valid = speechmarkdown_validate(inputPtr);
                    if (!valid)
                    {
                        string error = GetAndFreeError();
                        throw new SpeechMarkdownException(error ?? "Validation failed");
                    }
                    return true;
                }
                finally
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        public string ToSmd(string ssml)
        {
            if (ssml == null) throw new ArgumentNullException(nameof(ssml));

            lock (_lock)
            {
                IntPtr inputPtr = MarshalUtf8(ssml);

                try
                {
                    IntPtr resultPtr = speechmarkdown_to_smd(inputPtr);

                    if (resultPtr == IntPtr.Zero)
                    {
                        string error = GetAndFreeError();
                        throw new SpeechMarkdownException(error ?? "Unknown error converting SSML to SpeechMarkdown");
                    }

                    return PtrToStringAndFree(resultPtr);
                }
                finally
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        public string SupportedSsml(string platform)
        {
            if (platform == null) throw new ArgumentNullException(nameof(platform));

            lock (_lock)
            {
                IntPtr platformPtr = MarshalUtf8(platform);

                try
                {
                    IntPtr resultPtr = speechmarkdown_supported_ssml(platformPtr);

                    if (resultPtr == IntPtr.Zero)
                    {
                        string error = GetAndFreeError();
                        throw new SpeechMarkdownException(error ?? "Unknown error getting supported SSML");
                    }

                    return PtrToStringAndFree(resultPtr);
                }
                finally
                {
                    Marshal.FreeHGlobal(platformPtr);
                }
            }
        }

        private static IntPtr MarshalUtf8(string s)
        {
            byte[] bytes = Encoding.UTF8.GetBytes(s + "\0");
            IntPtr ptr = Marshal.AllocHGlobal(bytes.Length);
            Marshal.Copy(bytes, 0, ptr, bytes.Length);
            return ptr;
        }

        private static string PtrToStringAndFree(IntPtr ptr)
        {
            if (ptr == IntPtr.Zero) return null;

            int len = 0;
            while (Marshal.ReadByte(ptr, len) != 0) len++;

            byte[] bytes = new byte[len];
            Marshal.Copy(ptr, bytes, 0, len);
            speechmarkdown_free(ptr);

            return Encoding.UTF8.GetString(bytes);
        }

        private static string GetAndFreeError()
        {
            IntPtr errorPtr = speechmarkdown_get_error();
            return PtrToStringAndFree(errorPtr);
        }

        public void Dispose()
        {
            GC.SuppressFinalize(this);
        }
    }

    public class SpeechMarkdownException : Exception
    {
        public SpeechMarkdownException(string message) : base(message) { }
    }

    public static class Platform
    {
        public const string AmazonAlexa = "amazon-alexa";
        public const string GoogleAssistant = "google-assistant";
        public const string MicrosoftAzure = "microsoft-azure";
        public const string Apple = "apple";
        public const string W3c = "w3c";
        public const string SamsungBixby = "samsung-bixby";
        public const string ElevenLabs = "eleven-labs";
        public const string IbmWatson = "ibm-watson";
    }
}
