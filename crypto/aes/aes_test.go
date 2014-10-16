package aes

import (
	"os"
	"testing"
)

func TestGenerateSalt(t *testing.T) {
	salt, err := GenerateSalt()
	if err != nil {
		t.Errorf("GenerateSalt() returns error: ", err)
	}
	if len(salt) != saltSize {
		t.Errorf("Salt should be of length 8, it is ", len(salt))
	}
	salt2, err := GenerateSalt()
	if err != nil {
		t.Errorf("GenerateSalt() returns error: ", err)
	}
	e := true
	for i := 0; i < len(salt); i++ {
		if salt[i] != salt2[i] {
			e = false
		}
	}
	if e {
		t.Errorf("salt should not be the same as salt2")
	}
}

func TestMakeAES256Key(t *testing.T) {
	salt, err := GenerateSalt()
	if err != nil {
		t.Errorf("GenerateSalt() returned error: ", err)
	}
	aeskey, err := MakeAES256Key("test passphrase", salt)
	if err != nil {
		t.Errorf("MakeAES256Key() returned error: ", err)
	}
	if len(aeskey.key) != 32 {
		t.Errorf("AES key should be of length 32 (256-bit) it is not")
	}
	if len(aeskey.salt) != saltSize {
		t.Errorf("AES key salt should be of length 8, it is not")
	}
	aeskey2, err := MakeAES256Key("test passphrase", salt)
	if err != nil {
		t.Errorf("MakeAES256Key() returned error: ", err)
	}
	e := false
	for i := 0; i < len(aeskey.key); i++ {
		if aeskey.key[i] != aeskey2.key[i] {
			e = true
		}
	}
	if e {
		t.Errorf("MakeAES256Key() should be deterministic for same passphrase and salt")
	}
	aeskey3, err := MakeAES256Key("test passphrase", nil)
	if err != nil {
		t.Errorf("MakeAES256Key() returned error: ", err)
	}
	e = true
	for i := 0; i < len(aeskey.key); i++ {
		if aeskey.key[i] != aeskey3.key[i] {
			e = false
		}
	}
	if e {
		t.Errorf("MakeAES256Key() with nil salt parameter should generate new key")
	}
}

func TestEncryption(t *testing.T) {
	plainData := []byte("This is my super secret secret. Keep safe pls. Ty.")
	passphrase := "test passphrase"
	s, _ := GenerateSalt()

	aeskey, err := MakeAES256Key(passphrase, s)

	// Make sure there are now errors with MakeAES256Key()
	if err != nil {
		t.Errorf("MakeAES256Key() gave error: ", err)
	}

	msg, err := EncryptAES256(*aeskey, plainData)
	// Make sure there are no errors with EncryptAES256()
	if err != nil {
		t.Errorf("EncryptAES256() returned error: ", err)
	}
	ciphertext, err := ExtractMsg(msg)
	// Make sure there are no errors with ExtractMsg()
	if err != nil {
		t.Errorf("ExtractMsg() returned error: ", err)
	}
	plaintext, err := DecryptAES256(*aeskey, ciphertext)

	// Check that the length of plaintext and plainData are the same
	if len(plaintext) != len(plainData) {
		t.Errorf("plaintext should have same length as plainData")
	}

	// Check that plaintext and plainData are indeed idenitcal
	e := false
	for i := 0; i < len(plaintext); i++ {
		if plaintext[i] != plainData[i] {
			e = true
		}
	}
	if e {
		t.Errorf("Decryption should return plainData, it does not")
	}

}

func TestEncryptDecrypt(t *testing.T) {
	plainData := []byte("This is my super secret secret. Keep safe pls. Ty.")
	passphrase := "test passphrase"
	ciphertext, err := Encrypt(passphrase, plainData)
	if err != nil {
		t.Errorf("Encrypt() raised error: ", err)
	}
	plaintext, err := Decrypt(passphrase, ciphertext)
	if err != nil {
		t.Errorf("Decrypt() raised error: ", err)
	}
	if len(plaintext) != len(plainData) {
		t.Errorf("plainData and plaintext should have the same length")
	}
	same := true
	for i := 0; i < len(plainData); i++ {
		if plaintext[i] != plainData[i] {
			same = false
		}
	}
	if !same {
		t.Errorf("plainData and plaintext should be the same")
	}
}

func TestFileFunctions(t *testing.T) {
	plainData := []byte("This is my super secret secret. Keep safe pls. Ty.")
	passphrase := "test passphrase"
	key, err := MakeAES256Key(passphrase, nil)
	if err != nil {
		t.Errorf("MakeAES256Key() returned error: ", err)
	}
	f, err := OpenFile("test/myfile.aes", os.O_CREATE|os.O_RDWR, *key)
	if err != nil {
		t.Errorf("OpenFile() returned error: ", err)
	}
	_, err = f.Write(plainData)
	if err != nil {
		t.Errorf("AESFile.Write() returned error: ", err)
	}
	/* this is a bad test
	if n != len(plainData) {
		t.Errorf("return of AESFile.Write() should be equal to input length")
	}*/
	err = f.Close()
	if err != nil {
		t.Errorf("AESFile.Close() returned error: ", err)
	}
}

func TestExtractFunctions(t *testing.T) {
	salt, err := GenerateSalt()
	if err != nil {
		t.Errorf("GenerateSalt() returned error: ", err)
	}
	plainData := []byte("My secret message")
	passphrase := "test passphrase"
	key, err := MakeAES256Key(passphrase, salt)
	if err != nil {
		t.Errorf("MakeAES256Key() returned error: ", err)
	}
	msg, err := EncryptAES256(*key, plainData)
	if err != nil {
		t.Errorf("EncryptAES256() returned error: ", err)
	}
	new_salt, err := ExtractSalt(msg)
	if err != nil {
		t.Errorf("ExtractSalt() returned error: ", err)
	}
	salt_good := true
	for i := 0; i < len(salt); i++ {
		if salt[i] != new_salt[i] {
			salt_good = false
		}
	}
	if !salt_good {
		t.Errorf("ExtractSalt() return value should be equal to salt, it is not")
	}
	new_msg, err := ExtractMsg(msg)
	if err != nil {
		t.Errorf("ExtractMsg() returned error: ", err)
	}
	new_plainData, err := DecryptAES256(*key, new_msg)
	plainData_good := true
	for i := 0; i < len(plainData); i++ {
		if plainData[i] != new_plainData[i] {
			plainData_good = false
		}
	}
	if !plainData_good {
		t.Errorf("Decrypted return from ExtractMsg() should be equal to plainData, it is not")
	}
}
