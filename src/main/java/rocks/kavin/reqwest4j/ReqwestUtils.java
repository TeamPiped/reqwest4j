package rocks.kavin.reqwest4j;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

public class ReqwestUtils {

    static {
        String arch = switch (System.getProperty("os.arch")) {
            case "aarch64" -> "aarch64";
            case "amd64" -> "x86_64";
            default -> throw new RuntimeException("Unsupported architecture");
        };

        String os = System.getProperty("os.name").toLowerCase();

        String extension;
        String native_folder;

        if (os.contains("win")) {
            extension = ".dll";
            native_folder = "windows";
        } else if (os.contains("linux")) {
            extension = ".so";
            native_folder = "linux";
        } else if (os.contains("darwin")) {
            extension = ".dylib";
            native_folder = "darwin"; // or apple?
        } else {
            throw new RuntimeException("OS not supported");
        }

        File nativeFile;

        try {
            nativeFile = File.createTempFile("libreqwest", extension);
            nativeFile.deleteOnExit();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        final var cl = ReqwestUtils.class.getClassLoader();

        try (
                var stream = cl.getResourceAsStream("META-INF/natives/" + native_folder + "/" + arch + "/libreqwest" + extension);
                var fileOutputStream = new FileOutputStream(nativeFile)
        ) {
            stream.transferTo(fileOutputStream);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        System.load(nativeFile.getAbsolutePath());
    }

    public static native void init(String proxy, String user, String pass);

    public static native CompletableFuture<Response> fetch(String url, String method, byte[] body,
                                                           Map<String, String> headers);

}
